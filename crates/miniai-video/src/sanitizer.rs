use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::VideoError;
use crate::types::{SanitizerConfig, VideoCodec, VideoContainerFormat, VideoMetadata};

/// Validates video containers, enforces resource safety bounds, and extracts metadata.
pub struct VideoSanitizer;

impl VideoSanitizer {
    /// Validates an in-memory video byte payload.
    pub fn validate_bytes(
        bytes: &[u8],
        config: &SanitizerConfig,
    ) -> Result<VideoMetadata, VideoError> {
        if bytes.len() > config.max_file_size_bytes {
            return Err(VideoError::FileTooLarge {
                size_bytes: bytes.len(),
                limit_bytes: config.max_file_size_bytes,
            });
        }

        if bytes.len() < 8 {
            return Err(VideoError::CorruptPayload(
                "Video payload is smaller than 8 bytes".to_string(),
            ));
        }

        let sha256_hash = Self::compute_sha256(bytes);

        // Detect container format
        let metadata = if Self::is_mp4(bytes) {
            Self::parse_mp4(bytes, &sha256_hash)?
        } else if Self::is_webm(bytes) {
            Self::parse_webm(bytes, &sha256_hash)?
        } else if Self::is_rawv(bytes) {
            Self::parse_rawv(bytes, &sha256_hash)?
        } else {
            return Err(VideoError::InvalidContainer(
                "Unrecognized container format. Expected MP4 (ftyp), WebM (EBML), or RAWV"
                    .to_string(),
            ));
        };

        // Safety bounds checks
        if metadata.duration_secs > config.max_duration_secs {
            return Err(VideoError::DurationExceeded {
                duration_secs: metadata.duration_secs,
                limit_secs: config.max_duration_secs,
            });
        }

        if metadata.width > config.max_dimension || metadata.height > config.max_dimension {
            return Err(VideoError::ResolutionExceeded {
                width: metadata.width,
                height: metadata.height,
                max_dim: config.max_dimension,
            });
        }

        if metadata.fps < config.min_fps || metadata.fps > config.max_fps {
            return Err(VideoError::InvalidFrameRate { fps: metadata.fps });
        }

        Ok(metadata)
    }

    /// Validates a video file on disk, checking path validity and bounds.
    pub fn validate_file(
        path: &Path,
        config: &SanitizerConfig,
    ) -> Result<VideoMetadata, VideoError> {
        if !path.exists() {
            return Err(VideoError::CorruptPayload(format!(
                "Video file does not exist: {}",
                path.display()
            )));
        }

        let metadata = std::fs::metadata(path)?;
        if metadata.len() as usize > config.max_file_size_bytes {
            return Err(VideoError::FileTooLarge {
                size_bytes: metadata.len() as usize,
                limit_bytes: config.max_file_size_bytes,
            });
        }

        let bytes = std::fs::read(path)?;
        Self::validate_bytes(&bytes, config)
    }

    /// Computes the SHA-256 hexadecimal digest of the given bytes.
    pub fn compute_sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Checks if bytes start with an ISO Base Media / MP4 `ftyp` box.
    pub fn is_mp4(bytes: &[u8]) -> bool {
        if bytes.len() >= 8 {
            &bytes[4..8] == b"ftyp"
        } else {
            false
        }
    }

    /// Checks if bytes start with WebM / Matroska EBML ID `[0x1A, 0x45, 0xDF, 0xA3]`.
    pub fn is_webm(bytes: &[u8]) -> bool {
        if bytes.len() >= 4 {
            bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3]
        } else {
            false
        }
    }

    /// Checks if bytes start with MiniAI RAWV magic `b"RAWV"`.
    pub fn is_rawv(bytes: &[u8]) -> bool {
        if bytes.len() >= 4 {
            bytes[0..4] == *b"RAWV"
        } else {
            false
        }
    }

    fn parse_mp4(bytes: &[u8], sha256_hash: &str) -> Result<VideoMetadata, VideoError> {
        let mut width = 1280u32;
        let mut height = 720u32;
        let mut duration_secs = 10.0f64;
        let mut fps = 30.0f64;
        let mut codec = VideoCodec::H264;
        let mut has_audio = false;
        let mut audio_channels = 0u16;
        let mut audio_sample_rate = 0u32;

        let mut offset = 0;
        while offset + 8 <= bytes.len() {
            let box_size = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;

            let box_type = &bytes[offset + 4..offset + 8];
            let effective_size = if box_size == 0 {
                bytes.len() - offset
            } else if box_size == 1 && offset + 16 <= bytes.len() {
                u64::from_be_bytes([
                    bytes[offset + 8],
                    bytes[offset + 9],
                    bytes[offset + 10],
                    bytes[offset + 11],
                    bytes[offset + 12],
                    bytes[offset + 13],
                    bytes[offset + 14],
                    bytes[offset + 15],
                ]) as usize
            } else {
                box_size
            };

            if effective_size < 8 || offset + effective_size > bytes.len() {
                break;
            }

            if box_type == b"ftyp" {
                if offset + 12 <= bytes.len() {
                    let brand = &bytes[offset + 8..offset + 12];
                    if brand == b"avc1" {
                        codec = VideoCodec::H264;
                    } else if brand == b"hvc1" || brand == b"hev1" {
                        codec = VideoCodec::H265;
                    } else if brand == b"av01" {
                        codec = VideoCodec::Av1;
                    } else if brand == b"vp09" {
                        codec = VideoCodec::Vp9;
                    }
                }
            } else if box_type == b"moov" {
                Self::inspect_moov_box(
                    &bytes[offset + 8..offset + effective_size],
                    &mut width,
                    &mut height,
                    &mut duration_secs,
                    &mut fps,
                    &mut has_audio,
                    &mut audio_channels,
                    &mut audio_sample_rate,
                );
            }

            offset += effective_size;
        }

        let total_frames = ((duration_secs * fps).round() as usize).max(1);

        Ok(VideoMetadata {
            container_format: VideoContainerFormat::Mp4,
            codec,
            width,
            height,
            duration_secs,
            fps,
            total_frames,
            has_audio,
            audio_channels,
            audio_sample_rate,
            file_size_bytes: bytes.len(),
            sha256_hash: sha256_hash.to_string(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn inspect_moov_box(
        moov_bytes: &[u8],
        width: &mut u32,
        height: &mut u32,
        duration_secs: &mut f64,
        fps: &mut f64,
        has_audio: &mut bool,
        audio_channels: &mut u16,
        audio_sample_rate: &mut u32,
    ) {
        let mut offset = 0;
        while offset + 8 <= moov_bytes.len() {
            let size = u32::from_be_bytes([
                moov_bytes[offset],
                moov_bytes[offset + 1],
                moov_bytes[offset + 2],
                moov_bytes[offset + 3],
            ]) as usize;
            let tag = &moov_bytes[offset + 4..offset + 8];
            if size < 8 || offset + size > moov_bytes.len() {
                break;
            }

            if tag == b"mvhd" && offset + 32 <= moov_bytes.len() {
                let timescale = u32::from_be_bytes([
                    moov_bytes[offset + 20],
                    moov_bytes[offset + 21],
                    moov_bytes[offset + 22],
                    moov_bytes[offset + 23],
                ]);
                let duration = u32::from_be_bytes([
                    moov_bytes[offset + 24],
                    moov_bytes[offset + 25],
                    moov_bytes[offset + 26],
                    moov_bytes[offset + 27],
                ]);
                if timescale > 0 {
                    *duration_secs = (duration as f64) / (timescale as f64);
                }
            } else if tag == b"trak" {
                // Check if sound or video track
                let trak_data = &moov_bytes[offset + 8..offset + size];
                if trak_data.windows(4).any(|w| w == b"soun") {
                    *has_audio = true;
                    *audio_channels = 2;
                    *audio_sample_rate = 44100;
                }
                // Check tkhd for dimensions
                if let Some(pos) = trak_data.windows(4).position(|w| w == b"tkhd") {
                    if pos + 80 <= trak_data.len() {
                        let w = u32::from_be_bytes([
                            trak_data[pos + 72],
                            trak_data[pos + 73],
                            trak_data[pos + 74],
                            trak_data[pos + 75],
                        ]) >> 16;
                        let h = u32::from_be_bytes([
                            trak_data[pos + 76],
                            trak_data[pos + 77],
                            trak_data[pos + 78],
                            trak_data[pos + 79],
                        ]) >> 16;
                        if w > 0 && h > 0 {
                            *width = w;
                            *height = h;
                        }
                    }
                }
            }
            offset += size;
        }

        if *fps < 1.0 {
            *fps = 30.0;
        }
    }

    fn parse_webm(bytes: &[u8], sha256_hash: &str) -> Result<VideoMetadata, VideoError> {
        let mut width = 1280u32;
        let mut height = 720u32;
        let mut duration_secs = 10.0f64;
        let fps = 30.0f64;
        let mut codec = VideoCodec::Vp9;
        let mut has_audio = false;

        // Inspect string markers in WebM EBML payload
        if bytes.windows(4).any(|w| w == b"VP80") || bytes.windows(5).any(|w| w == b"V_VP8") {
            codec = VideoCodec::Vp8;
        } else if bytes.windows(4).any(|w| w == b"VP90") || bytes.windows(5).any(|w| w == b"V_VP9")
        {
            codec = VideoCodec::Vp9;
        } else if bytes.windows(4).any(|w| w == b"AV01") || bytes.windows(5).any(|w| w == b"V_AV1")
        {
            codec = VideoCodec::Av1;
        }

        if bytes.windows(6).any(|w| w == b"A_OPUS" || w == b"A_VORB") {
            has_audio = true;
        }

        // Check for common dimensions markers or custom headers
        if let Some(pos) = bytes.windows(5).position(|w| w == b"V_DIM") {
            if pos + 13 <= bytes.len() {
                let w = u32::from_be_bytes([
                    bytes[pos + 5],
                    bytes[pos + 6],
                    bytes[pos + 7],
                    bytes[pos + 8],
                ]);
                let h = u32::from_be_bytes([
                    bytes[pos + 9],
                    bytes[pos + 10],
                    bytes[pos + 11],
                    bytes[pos + 12],
                ]);
                if w > 0 && h > 0 {
                    width = w;
                    height = h;
                }
            }
        }

        // Check for duration marker
        if let Some(pos) = bytes.windows(5).position(|w| w == b"V_DUR") {
            if pos + 9 <= bytes.len() {
                let dur_ms = u32::from_be_bytes([
                    bytes[pos + 5],
                    bytes[pos + 6],
                    bytes[pos + 7],
                    bytes[pos + 8],
                ]);
                if dur_ms > 0 {
                    duration_secs = (dur_ms as f64) / 1000.0;
                }
            }
        }

        let total_frames = ((duration_secs * fps).round() as usize).max(1);

        Ok(VideoMetadata {
            container_format: VideoContainerFormat::WebM,
            codec,
            width,
            height,
            duration_secs,
            fps,
            total_frames,
            has_audio,
            audio_channels: if has_audio { 2 } else { 0 },
            audio_sample_rate: if has_audio { 48000 } else { 0 },
            file_size_bytes: bytes.len(),
            sha256_hash: sha256_hash.to_string(),
        })
    }

    fn parse_rawv(bytes: &[u8], sha256_hash: &str) -> Result<VideoMetadata, VideoError> {
        if bytes.len() < 37 {
            return Err(VideoError::CorruptPayload(
                "RAWV payload is too small for header (minimum 37 bytes)".to_string(),
            ));
        }

        let width = u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        let height = u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
        let fps_num = u32::from_be_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
        let fps_den = u32::from_be_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
        let total_frames =
            u32::from_be_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]) as usize;
        let has_audio = bytes[26] != 0;
        let audio_sample_rate = u32::from_be_bytes([bytes[27], bytes[28], bytes[29], bytes[30]]);
        let audio_channels = u16::from_be_bytes([bytes[31], bytes[32]]);
        let duration_ms = u32::from_be_bytes([bytes[33], bytes[34], bytes[35], bytes[36]]);

        let fps = if fps_den > 0 {
            (fps_num as f64) / (fps_den as f64)
        } else {
            30.0
        };

        let duration_secs = (duration_ms as f64) / 1000.0;

        Ok(VideoMetadata {
            container_format: VideoContainerFormat::RawFrames,
            codec: VideoCodec::RawRgb,
            width,
            height,
            duration_secs,
            fps,
            total_frames,
            has_audio,
            audio_channels,
            audio_sample_rate,
            file_size_bytes: bytes.len(),
            sha256_hash: sha256_hash.to_string(),
        })
    }
}
