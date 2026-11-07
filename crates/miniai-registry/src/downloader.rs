use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error::RegistryError;
use crate::models::{DownloadProgress, DownloadStatus, ModelManifest};

/// Resumable HTTP and file downloader with mandatory cryptographic SHA-256 verification.
pub struct ResumableDownloader;

impl ResumableDownloader {
    /// Computes the SHA-256 hexadecimal digest of a file on disk.
    pub fn compute_file_sha256(path: &Path) -> Result<String, RegistryError> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{result:x}"))
    }

    /// Downloads or copies a model into staging, verifying SHA-256 before atomic promotion.
    pub async fn download<F>(
        manifest: &ModelManifest,
        staging_dir: &Path,
        final_dir: &Path,
        mut on_progress: F,
    ) -> Result<PathBuf, RegistryError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        manifest.validate()?;

        std::fs::create_dir_all(staging_dir)?;
        std::fs::create_dir_all(final_dir)?;

        let staging_file = staging_dir.join(format!("{}.part", manifest.model_id));
        let final_file = final_dir.join(format!("{}.{}", manifest.model_id, manifest.format));

        // If final file already exists and matches expected SHA-256, return immediately
        if final_file.is_file() {
            let existing_sha = Self::compute_file_sha256(&final_file)?;
            if existing_sha.eq_ignore_ascii_case(&manifest.expected_sha256) {
                on_progress(DownloadProgress::new(
                    &manifest.model_id,
                    manifest.expected_size_bytes,
                    manifest.expected_size_bytes,
                    DownloadStatus::Verified,
                ));
                return Ok(final_file);
            }
        }

        // Perform download into staging file
        if manifest.source_url.starts_with("http://") || manifest.source_url.starts_with("https://")
        {
            Self::download_http(manifest, &staging_file, &mut on_progress).await?;
        } else if manifest.source_url.starts_with("file://")
            || Path::new(&manifest.source_url).exists()
        {
            Self::download_local_file(manifest, &staging_file, &mut on_progress).await?;
        } else {
            return Err(RegistryError::DownloadError {
                model_id: manifest.model_id.clone(),
                message: format!(
                    "Unsupported protocol in source_url: '{}'",
                    manifest.source_url
                ),
            });
        }

        // Transition to verifying status
        on_progress(DownloadProgress::new(
            &manifest.model_id,
            manifest.expected_size_bytes,
            manifest.expected_size_bytes,
            DownloadStatus::VerifyingSha256,
        ));

        // Calculate actual SHA-256 digest of the staged artifact
        let actual_sha = Self::compute_file_sha256(&staging_file)?;

        if !actual_sha.eq_ignore_ascii_case(&manifest.expected_sha256) {
            // Delete corrupt or tampered staging file immediately
            let _ = std::fs::remove_file(&staging_file);
            on_progress(DownloadProgress::new(
                &manifest.model_id,
                0,
                manifest.expected_size_bytes,
                DownloadStatus::Failed {
                    error: format!(
                        "SHA-256 mismatch: expected {}, computed {}",
                        manifest.expected_sha256, actual_sha
                    ),
                },
            ));
            return Err(RegistryError::Sha256Mismatch {
                model_id: manifest.model_id.clone(),
                expected: manifest.expected_sha256.clone(),
                actual: actual_sha,
            });
        }

        // Atomic promotion from staging to verified model directory
        std::fs::rename(&staging_file, &final_file)?;

        on_progress(DownloadProgress::new(
            &manifest.model_id,
            manifest.expected_size_bytes,
            manifest.expected_size_bytes,
            DownloadStatus::Verified,
        ));

        Ok(final_file)
    }

    /// Resumable HTTP download supporting `Range: bytes={offset}-`.
    async fn download_http<F>(
        manifest: &ModelManifest,
        staging_file: &Path,
        on_progress: &mut F,
    ) -> Result<(), RegistryError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let url = &manifest.source_url;
        let is_https = url.starts_with("https://");
        let stripped = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        let (host_port, path) = match stripped.split_once('/') {
            Some((hp, p)) => (hp, format!("/{p}")),
            None => (stripped, "/".to_string()),
        };

        let target_addr = if host_port.contains(':') {
            host_port.to_string()
        } else if is_https {
            format!("{host_port}:443")
        } else {
            format!("{host_port}:80")
        };

        // Determine existing bytes for HTTP Range request
        let mut existing_bytes = if staging_file.is_file() {
            std::fs::metadata(staging_file)?.len()
        } else {
            0
        };

        if existing_bytes > manifest.expected_size_bytes {
            // Staging file is larger than expected; truncate and restart
            let _ = std::fs::remove_file(staging_file);
            existing_bytes = 0;
        }

        let mut stream =
            TcpStream::connect(&target_addr)
                .await
                .map_err(|e| RegistryError::DownloadError {
                    model_id: manifest.model_id.clone(),
                    message: format!("Failed to connect to HTTP host '{target_addr}': {e}"),
                })?;

        let mut request = format!(
            "GET {path} HTTP/1.1\r\n\
             Host: {host_port}\r\n\
             User-Agent: MiniAI-Registry/0.1\r\n\
             Connection: close\r\n"
        );

        if existing_bytes > 0 {
            request.push_str(&format!("Range: bytes={existing_bytes}-\r\n"));
        }
        request.push_str("\r\n");

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| RegistryError::DownloadError {
                model_id: manifest.model_id.clone(),
                message: format!("Failed to send HTTP request: {e}"),
            })?;

        // Read response header and body
        let mut header_buf = Vec::new();
        let mut byte = [0u8; 1];

        while header_buf.len() < 8192 {
            let n = stream.read(&mut byte).await?;
            if n == 0 {
                break;
            }
            header_buf.push(byte[0]);
            if header_buf.ends_with(b"\r\n\r\n") {
                break;
            }
        }

        let header_str = String::from_utf8_lossy(&header_buf);
        let status_code = Self::parse_http_status(&header_str)?;

        let mut file = if status_code == 206 {
            // Append to existing staging file
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(staging_file)?
        } else if status_code == 200 {
            // Server ignored Range header or fresh download; start from 0
            existing_bytes = 0;
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(staging_file)?
        } else {
            return Err(RegistryError::HttpError {
                status: status_code,
                message: format!("Unexpected HTTP response: {header_str}"),
            });
        };

        let mut current_bytes = existing_bytes;
        let mut chunk_buf = [0u8; 65536];

        on_progress(DownloadProgress::new(
            &manifest.model_id,
            current_bytes,
            manifest.expected_size_bytes,
            DownloadStatus::Downloading {
                bytes_downloaded: current_bytes,
                total_bytes: manifest.expected_size_bytes,
            },
        ));

        loop {
            let n = stream.read(&mut chunk_buf).await?;
            if n == 0 {
                break;
            }
            file.write_all(&chunk_buf[..n])?;
            current_bytes += n as u64;

            on_progress(DownloadProgress::new(
                &manifest.model_id,
                current_bytes,
                manifest.expected_size_bytes,
                DownloadStatus::Downloading {
                    bytes_downloaded: current_bytes,
                    total_bytes: manifest.expected_size_bytes,
                },
            ));
        }

        file.flush()?;
        Ok(())
    }

    /// Local file copy simulating chunked range downloads for offline validation.
    async fn download_local_file<F>(
        manifest: &ModelManifest,
        staging_file: &Path,
        on_progress: &mut F,
    ) -> Result<(), RegistryError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let local_path_str = manifest
            .source_url
            .strip_prefix("file://")
            .unwrap_or(&manifest.source_url);
        let src_path = Path::new(local_path_str);

        if !src_path.is_file() {
            return Err(RegistryError::DownloadError {
                model_id: manifest.model_id.clone(),
                message: format!("Local source file does not exist: '{}'", src_path.display()),
            });
        }

        let mut src_file = std::fs::File::open(src_path)?;
        let src_len = src_file.metadata()?.len();

        let mut existing_bytes = if staging_file.is_file() {
            std::fs::metadata(staging_file)?.len()
        } else {
            0
        };

        if existing_bytes > src_len {
            let _ = std::fs::remove_file(staging_file);
            existing_bytes = 0;
        }

        src_file.seek(SeekFrom::Start(existing_bytes))?;

        let mut dest_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(staging_file)?;

        let mut current_bytes = existing_bytes;
        let mut buffer = [0u8; 65536];

        on_progress(DownloadProgress::new(
            &manifest.model_id,
            current_bytes,
            manifest.expected_size_bytes,
            DownloadStatus::Downloading {
                bytes_downloaded: current_bytes,
                total_bytes: manifest.expected_size_bytes,
            },
        ));

        loop {
            let n = src_file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            dest_file.write_all(&buffer[..n])?;
            current_bytes += n as u64;

            on_progress(DownloadProgress::new(
                &manifest.model_id,
                current_bytes,
                manifest.expected_size_bytes,
                DownloadStatus::Downloading {
                    bytes_downloaded: current_bytes,
                    total_bytes: manifest.expected_size_bytes,
                },
            ));
        }

        dest_file.flush()?;
        Ok(())
    }

    fn parse_http_status(header: &str) -> Result<u16, RegistryError> {
        let first_line = header.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(code) = parts[1].parse::<u16>() {
                return Ok(code);
            }
        }
        Err(RegistryError::HttpError {
            status: 0,
            message: format!("Malformed HTTP status line: '{first_line}'"),
        })
    }
}
