use sha2::{Digest, Sha256};

use crate::error::VisionError;
use crate::types::{ImageDimensions, ImageFormat, ImageMetadata};

/// Maximum allowable image file size in bytes (25 MB).
pub const MAX_IMAGE_FILE_SIZE_BYTES: usize = 25 * 1024 * 1024;

/// Maximum allowable dimension along either axis (4096 px).
pub const MAX_IMAGE_DIMENSION: u32 = 4096;

/// Maximum total pixels allowed (16 Megapixels).
pub const MAX_IMAGE_TOTAL_PIXELS: u64 = 4096 * 4096;

/// Minimum byte length to form any valid image header.
pub const MIN_IMAGE_HEADER_BYTES: usize = 12;

/// Sanitizer verifying image headers, magic bytes, dimension bounds, and anti-bomb constraints.
pub struct ImageSanitizer;

impl ImageSanitizer {
    /// Sanitizes raw image bytes, validating headers, bounds, and computing cryptographic digest.
    pub fn sanitize(bytes: &[u8]) -> Result<ImageMetadata, VisionError> {
        let size_bytes = bytes.len();
        if size_bytes < MIN_IMAGE_HEADER_BYTES {
            return Err(VisionError::CorruptPayload {
                details: format!("Image payload too short: {size_bytes} bytes"),
            });
        }

        if size_bytes > MAX_IMAGE_FILE_SIZE_BYTES {
            return Err(VisionError::FileTooLarge {
                size_bytes,
                max_bytes: MAX_IMAGE_FILE_SIZE_BYTES,
            });
        }

        let format = Self::detect_format(bytes)?;
        let (dimensions, has_alpha) = match format {
            ImageFormat::Png => Self::parse_png_header(bytes)?,
            ImageFormat::Jpeg => Self::parse_jpeg_header(bytes)?,
            ImageFormat::WebP => Self::parse_webp_header(bytes)?,
        };

        if dimensions.width == 0 || dimensions.height == 0 {
            return Err(VisionError::CorruptPayload {
                details: format!(
                    "Invalid non-positive dimensions: {}x{}",
                    dimensions.width, dimensions.height
                ),
            });
        }

        if dimensions.width > MAX_IMAGE_DIMENSION || dimensions.height > MAX_IMAGE_DIMENSION {
            return Err(VisionError::DimensionsExceeded {
                width: dimensions.width,
                height: dimensions.height,
                max_dim: MAX_IMAGE_DIMENSION,
            });
        }

        let total_pixels = dimensions.total_pixels();
        if total_pixels > MAX_IMAGE_TOTAL_PIXELS {
            return Err(VisionError::DecompressionBomb {
                total_pixels,
                max_pixels: MAX_IMAGE_TOTAL_PIXELS,
            });
        }

        // Decompression bomb defense: excessive pixel expansion relative to compressed payload size
        // If claimed uncompressed pixels exceed 250x file size and total pixels > 2 million, flag bomb
        if total_pixels > (size_bytes as u64) * 250 && total_pixels > 2_000_000 {
            return Err(VisionError::DecompressionBomb {
                total_pixels,
                max_pixels: (size_bytes as u64) * 250,
            });
        }

        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(ImageMetadata {
            format,
            dimensions,
            size_bytes,
            sha256,
            has_alpha,
        })
    }

    /// Detects image format based strictly on magic bytes.
    pub fn detect_format(bytes: &[u8]) -> Result<ImageFormat, VisionError> {
        if bytes.len() < 4 {
            return Err(VisionError::CorruptPayload {
                details: "Payload too small to contain magic bytes".to_string(),
            });
        }

        // PNG signature: 89 50 4E 47 0D 0A 1A 0A
        if bytes.len() >= 8 && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return Ok(ImageFormat::Png);
        }

        // JPEG signature: FF D8 FF
        if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
            return Ok(ImageFormat::Jpeg);
        }

        // WebP signature: RIFF....WEBP
        if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            return Ok(ImageFormat::WebP);
        }

        let detected_hex = bytes[..bytes.len().min(8)]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");

        Err(VisionError::InvalidMagicBytes {
            detected: detected_hex,
        })
    }

    /// Extracts dimensions and alpha channel status from PNG IHDR chunk.
    fn parse_png_header(bytes: &[u8]) -> Result<(ImageDimensions, bool), VisionError> {
        // PNG header: 8 bytes magic + 4 bytes length + 4 bytes "IHDR" + 13 bytes data
        if bytes.len() < 29 {
            return Err(VisionError::CorruptPayload {
                details: "Truncated PNG header: missing IHDR chunk".to_string(),
            });
        }

        if &bytes[12..16] != b"IHDR" {
            return Err(VisionError::CorruptPayload {
                details: "Expected IHDR as first PNG chunk".to_string(),
            });
        }

        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        let color_type = bytes[25];
        // PNG color types: 4 (grayscale+alpha), 6 (RGBA)
        let has_alpha = color_type == 4 || color_type == 6;

        Ok((ImageDimensions::new(width, height), has_alpha))
    }

    /// Extracts dimensions from JPEG Start of Frame (SOF) marker segments.
    fn parse_jpeg_header(bytes: &[u8]) -> Result<(ImageDimensions, bool), VisionError> {
        let mut offset = 2; // after 0xFF 0xD8
        let len = bytes.len();

        while offset + 4 < len {
            if bytes[offset] != 0xFF {
                offset += 1;
                continue;
            }

            let marker = bytes[offset + 1];
            offset += 2;

            // Standalone markers without length: SOI (0xD8), EOI (0xD9), RST0-7 (0xD0-0xD7)
            if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
                continue;
            }

            if offset + 2 > len {
                break;
            }

            let segment_len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            if segment_len < 2 || offset + segment_len > len {
                return Err(VisionError::CorruptPayload {
                    details: "Invalid JPEG segment length".to_string(),
                });
            }

            // SOF markers: SOF0 (0xC0), SOF1 (0xC1), SOF2 (0xC2), SOF3 (0xC3),
            // SOF5 (0xC5), SOF6 (0xC6), SOF7 (0xC7), SOF9 (0xC9), SOF10 (0xCA), SOF11 (0xCB)
            let is_sof = matches!(
                marker,
                0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB
            );

            if is_sof {
                if segment_len < 7 || offset + 7 > len {
                    return Err(VisionError::CorruptPayload {
                        details: "Truncated JPEG SOF segment".to_string(),
                    });
                }
                let height = u16::from_be_bytes([bytes[offset + 3], bytes[offset + 4]]) as u32;
                let width = u16::from_be_bytes([bytes[offset + 5], bytes[offset + 6]]) as u32;
                return Ok((ImageDimensions::new(width, height), false));
            }

            offset += segment_len;
        }

        Err(VisionError::CorruptPayload {
            details: "No Start of Frame (SOF) marker found in JPEG payload".to_string(),
        })
    }

    /// Extracts dimensions and alpha channel status from WebP chunk headers.
    fn parse_webp_header(bytes: &[u8]) -> Result<(ImageDimensions, bool), VisionError> {
        if bytes.len() < 24 {
            return Err(VisionError::CorruptPayload {
                details: "Truncated WebP header".to_string(),
            });
        }

        let chunk_type = &bytes[12..16];
        if chunk_type == b"VP8 " {
            // Lossy VP8 bitstream: offset 20..23 has keyframe sync code 0x9D 0x01 0x2A
            if bytes.len() < 30 {
                return Err(VisionError::CorruptPayload {
                    details: "Truncated WebP VP8 chunk".to_string(),
                });
            }
            if bytes[23] != 0x9D || bytes[24] != 0x01 || bytes[25] != 0x2A {
                return Err(VisionError::CorruptPayload {
                    details: "Invalid VP8 keyframe sync code".to_string(),
                });
            }
            let width = u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3FFF;
            let height = u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3FFF;
            Ok((ImageDimensions::new(width as u32, height as u32), false))
        } else if chunk_type == b"VP8L" {
            // Lossless VP8L bitstream
            if bytes.len() < 25 {
                return Err(VisionError::CorruptPayload {
                    details: "Truncated WebP VP8L chunk".to_string(),
                });
            }
            if bytes[20] != 0x2F {
                return Err(VisionError::CorruptPayload {
                    details: "Invalid VP8L signature byte".to_string(),
                });
            }
            let b1 = bytes[21] as u32;
            let b2 = bytes[22] as u32;
            let b3 = bytes[23] as u32;
            let b4 = bytes[24] as u32;

            let width = (b1 | ((b2 & 0x3F) << 8)) + 1;
            let height = ((b2 >> 6) | (b3 << 2) | ((b4 & 0x0F) << 10)) + 1;
            let has_alpha = (b4 & 0x10) != 0;
            Ok((ImageDimensions::new(width, height), has_alpha))
        } else if chunk_type == b"VP8X" {
            // Extended WebP
            if bytes.len() < 30 {
                return Err(VisionError::CorruptPayload {
                    details: "Truncated WebP VP8X chunk".to_string(),
                });
            }
            let flags = bytes[20];
            let has_alpha = (flags & 0x10) != 0;

            let w1 = bytes[24] as u32;
            let w2 = bytes[25] as u32;
            let w3 = bytes[26] as u32;
            let width = (w1 | (w2 << 8) | (w3 << 16)) + 1;

            let h1 = bytes[27] as u32;
            let h2 = bytes[28] as u32;
            let h3 = bytes[29] as u32;
            let height = (h1 | (h2 << 8) | (h3 << 16)) + 1;

            Ok((ImageDimensions::new(width, height), has_alpha))
        } else {
            Err(VisionError::CorruptPayload {
                details: format!(
                    "Unsupported WebP chunk variant: {:?}",
                    String::from_utf8_lossy(chunk_type)
                ),
            })
        }
    }
}
