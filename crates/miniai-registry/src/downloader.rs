use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error::RegistryError;
use crate::models::{DownloadProgress, DownloadStatus, ModelManifest};

/// Maximum number of HTTP redirects followed for a single artifact download.
const MAX_REDIRECTS: usize = 8;

/// Object-safe duplex stream alias covering both plain TCP and rustls TLS streams.
trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncStream for T {}

/// Minimal parsed `http(s)://host[:port]/path` target.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedUrl {
    https: bool,
    host: String,
    port: u16,
    host_header: String,
    path: String,
}

impl ParsedUrl {
    fn parse(model_id: &str, url: &str) -> Result<Self, RegistryError> {
        let bad = |msg: &str| RegistryError::DownloadError {
            model_id: model_id.to_string(),
            message: format!("{msg}: '{url}'"),
        };
        let (https, rest) = if let Some(r) = url.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = url.strip_prefix("http://") {
            (false, r)
        } else {
            return Err(bad("Unsupported URL scheme"));
        };
        let (host_port, path) = match rest.split_once('/') {
            Some((hp, p)) => (hp, format!("/{p}")),
            None => (rest, "/".to_string()),
        };
        if host_port.is_empty() {
            return Err(bad("Missing host in URL"));
        }
        let default_port = if https { 443 } else { 80 };
        let (host, port) = match host_port.rsplit_once(':') {
            Some((h, p)) => (
                h.to_string(),
                p.parse::<u16>().map_err(|_| bad("Invalid port in URL"))?,
            ),
            None => (host_port.to_string(), default_port),
        };
        Ok(Self {
            https,
            host,
            port,
            host_header: host_port.to_string(),
            path,
        })
    }

    /// Resolves an absolute or origin-relative `Location` header against this URL.
    fn resolve_redirect(&self, model_id: &str, location: &str) -> Result<Self, RegistryError> {
        if location.starts_with("http://") || location.starts_with("https://") {
            Self::parse(model_id, location)
        } else if location.starts_with('/') {
            let mut next = self.clone();
            next.path = location.to_string();
            Ok(next)
        } else {
            Err(RegistryError::DownloadError {
                model_id: model_id.to_string(),
                message: format!("Unsupported redirect Location '{location}'"),
            })
        }
    }
}

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

    /// Resumable HTTP(S) download supporting `Range: bytes={offset}-`, TLS 1.2/1.3 (rustls,
    /// Mozilla root store) and up to `MAX_REDIRECTS` HTTP 301/302/303/307/308 redirects
    /// (HuggingFace `resolve/main/...` URLs always redirect to a signed CDN URL).
    async fn download_http<F>(
        manifest: &ModelManifest,
        staging_file: &Path,
        on_progress: &mut F,
    ) -> Result<(), RegistryError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let model_id = manifest.model_id.as_str();
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

        let mut target = ParsedUrl::parse(model_id, &manifest.source_url)?;
        let mut redirects = 0usize;

        let (mut stream, status_code, header_str) = loop {
            let mut stream = Self::connect(model_id, &target).await?;

            let mut request = format!(
                "GET {} HTTP/1.1\r\n\
                 Host: {}\r\n\
                 User-Agent: MiniAI-Registry/0.1\r\n\
                 Accept: */*\r\n\
                 Accept-Encoding: identity\r\n\
                 Connection: close\r\n",
                target.path, target.host_header
            );
            if existing_bytes > 0 {
                request.push_str(&format!("Range: bytes={existing_bytes}-\r\n"));
            }
            request.push_str("\r\n");

            stream.write_all(request.as_bytes()).await.map_err(|e| {
                RegistryError::DownloadError {
                    model_id: model_id.to_string(),
                    message: format!("Failed to send HTTP request: {e}"),
                }
            })?;

            // Read response header (byte-wise so the body stays unread in the stream)
            let mut header_buf = Vec::new();
            let mut byte = [0u8; 1];
            while header_buf.len() < 16384 {
                let n = stream.read(&mut byte).await?;
                if n == 0 {
                    break;
                }
                header_buf.push(byte[0]);
                if header_buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }

            let header_str = String::from_utf8_lossy(&header_buf).to_string();
            let status_code = Self::parse_http_status(&header_str)?;

            if matches!(status_code, 301 | 302 | 303 | 307 | 308) {
                redirects += 1;
                if redirects > MAX_REDIRECTS {
                    return Err(RegistryError::HttpError {
                        status: status_code,
                        message: format!("Too many redirects (>{MAX_REDIRECTS})"),
                    });
                }
                let location = Self::header_value(&header_str, "location").ok_or_else(|| {
                    RegistryError::HttpError {
                        status: status_code,
                        message: "Redirect response missing Location header".to_string(),
                    }
                })?;
                target = target.resolve_redirect(model_id, &location)?;
                continue;
            }
            break (stream, status_code, header_str);
        };

        if Self::header_value(&header_str, "transfer-encoding")
            .map(|v| v.to_ascii_lowercase().contains("chunked"))
            .unwrap_or(false)
        {
            return Err(RegistryError::DownloadError {
                model_id: model_id.to_string(),
                message: "Chunked transfer encoding is not supported for model artifacts"
                    .to_string(),
            });
        }

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
        let mut chunk_buf = vec![0u8; 65536];

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
            let n = match stream.read(&mut chunk_buf).await {
                Ok(n) => n,
                // Some CDNs close TLS without close_notify after the final byte.
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => 0,
                Err(e) => return Err(e.into()),
            };
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

    /// Opens a plain TCP or rustls-encrypted connection to the target.
    async fn connect(
        model_id: &str,
        target: &ParsedUrl,
    ) -> Result<Box<dyn AsyncStream>, RegistryError> {
        let addr = format!("{}:{}", target.host, target.port);
        let tcp = TcpStream::connect(&addr)
            .await
            .map_err(|e| RegistryError::DownloadError {
                model_id: model_id.to_string(),
                message: format!("Failed to connect to HTTP host '{addr}': {e}"),
            })?;

        if !target.https {
            return Ok(Box::new(tcp));
        }

        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|e| RegistryError::DownloadError {
            model_id: model_id.to_string(),
            message: format!("TLS configuration error: {e}"),
        })?
        .with_root_certificates(roots)
        .with_no_client_auth();

        let server_name =
            rustls::pki_types::ServerName::try_from(target.host.clone()).map_err(|e| {
                RegistryError::DownloadError {
                    model_id: model_id.to_string(),
                    message: format!("Invalid TLS server name '{}': {e}", target.host),
                }
            })?;

        let tls = tokio_rustls::TlsConnector::from(Arc::new(config))
            .connect(server_name, tcp)
            .await
            .map_err(|e| RegistryError::DownloadError {
                model_id: model_id.to_string(),
                message: format!("TLS handshake with '{}' failed: {e}", target.host),
            })?;
        Ok(Box::new(tls))
    }

    /// Case-insensitive lookup of a response header value.
    fn header_value(header: &str, name: &str) -> Option<String> {
        header.lines().skip(1).find_map(|line| {
            let (k, v) = line.split_once(':')?;
            if k.trim().eq_ignore_ascii_case(name) {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
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
