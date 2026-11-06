use std::collections::HashMap;

use crate::error::ImageGenError;
use crate::types::ImageProvenance;

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const MAX_DEFLATE_BLOCK_SIZE: usize = 65535;

/// Precomputed CRC-32 lookup table (polynomial 0xEDB88320).
const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};

/// Computes CRC-32 over input slices.
pub fn calculate_crc32(slices: &[&[u8]]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for slice in slices {
        for &byte in *slice {
            let table_idx = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = CRC_TABLE[table_idx] ^ (crc >> 8);
        }
    }
    !crc
}

/// Computes Adler-32 checksum (RFC 1950) for zlib data integrity.
pub fn calculate_adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut s1 = 1u32;
    let mut s2 = 0u32;

    for &byte in data {
        s1 = (s1 + byte as u32) % MOD_ADLER;
        s2 = (s2 + s1) % MOD_ADLER;
    }

    (s2 << 16) | s1
}

/// Lossless PNG encoder embedding standard chunks and cryptographic provenance metadata.
pub struct PngEncoder;

impl PngEncoder {
    /// Encodes raw 24-bit RGB pixel data into a standard lossless PNG with embedded provenance.
    pub fn encode(
        rgb: &[u8],
        width: u32,
        height: u32,
        provenance: &ImageProvenance,
    ) -> Result<Vec<u8>, ImageGenError> {
        let expected_bytes = (width as usize) * (height as usize) * 3;
        if rgb.len() < expected_bytes {
            return Err(ImageGenError::PngError(format!(
                "RGB buffer underflow: expected {} bytes, got {}",
                expected_bytes,
                rgb.len()
            )));
        }

        let mut output = Vec::with_capacity(expected_bytes + 2048);
        output.extend_from_slice(&PNG_SIGNATURE);

        // 1. IHDR Chunk
        let mut ihdr_data = Vec::with_capacity(13);
        ihdr_data.extend_from_slice(&width.to_be_bytes());
        ihdr_data.extend_from_slice(&height.to_be_bytes());
        ihdr_data.push(8); // Bit depth: 8
        ihdr_data.push(2); // Color type: 2 (Truecolor RGB)
        ihdr_data.push(0); // Compression: Deflate
        ihdr_data.push(0); // Filter: Adaptive (0)
        ihdr_data.push(0); // Interlace: None (0)
        Self::write_chunk(&mut output, b"IHDR", &ihdr_data);

        // 2. tEXt Provenance Chunks
        Self::write_text_chunk(&mut output, "miniai:prompt", &provenance.prompt);
        if let Some(ref neg) = provenance.negative_prompt {
            Self::write_text_chunk(&mut output, "miniai:negative_prompt", neg);
        }
        Self::write_text_chunk(&mut output, "miniai:seed", &provenance.seed.to_string());
        Self::write_text_chunk(&mut output, "miniai:steps", &provenance.steps.to_string());
        Self::write_text_chunk(
            &mut output,
            "miniai:guidance_scale",
            &format!("{:.2}", provenance.guidance_scale),
        );
        Self::write_text_chunk(
            &mut output,
            "miniai:dimensions",
            &format!("{}x{}", provenance.width, provenance.height),
        );
        Self::write_text_chunk(&mut output, "miniai:sha256", &provenance.sha256);
        Self::write_text_chunk(&mut output, "miniai:model_id", &provenance.model_id);
        Self::write_text_chunk(&mut output, "miniai:backend", &provenance.backend);
        Self::write_text_chunk(
            &mut output,
            "miniai:timestamp",
            &provenance.created_at_epoch_secs.to_string(),
        );

        // 3. Prepare scanlines with filter type 0 (None)
        let row_stride = (width as usize) * 3;
        let scanlines_len = (height as usize) * (1 + row_stride);
        let mut raw_scanlines = Vec::with_capacity(scanlines_len);

        for y in 0..(height as usize) {
            raw_scanlines.push(0); // Filter byte: 0 (None)
            let start = y * row_stride;
            let end = start + row_stride;
            raw_scanlines.extend_from_slice(&rgb[start..end]);
        }

        // 4. Zlib packaging with non-compressed DEFLATE blocks (BTYPE 00)
        let mut zlib_data = Vec::with_capacity(scanlines_len + 128);
        zlib_data.push(0x78); // CMF (Deflate, 32K window)
        zlib_data.push(0x01); // FLG (check bits)

        let chunks = raw_scanlines.chunks(MAX_DEFLATE_BLOCK_SIZE);
        let total_chunks = chunks.len();

        for (idx, chunk) in chunks.enumerate() {
            let is_last = idx == total_chunks - 1;
            let header = if is_last { 0x01 } else { 0x00 };
            zlib_data.push(header);

            let len = chunk.len() as u16;
            let nlen = !len;
            zlib_data.extend_from_slice(&len.to_le_bytes());
            zlib_data.extend_from_slice(&nlen.to_le_bytes());
            zlib_data.extend_from_slice(chunk);
        }

        let adler = calculate_adler32(&raw_scanlines);
        zlib_data.extend_from_slice(&adler.to_be_bytes());

        // Write IDAT chunk
        Self::write_chunk(&mut output, b"IDAT", &zlib_data);

        // 5. IEND Chunk
        Self::write_chunk(&mut output, b"IEND", &[]);

        Ok(output)
    }

    fn write_chunk(output: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        let length = data.len() as u32;
        output.extend_from_slice(&length.to_be_bytes());
        output.extend_from_slice(chunk_type);
        output.extend_from_slice(data);

        let crc = calculate_crc32(&[chunk_type, data]);
        output.extend_from_slice(&crc.to_be_bytes());
    }

    fn write_text_chunk(output: &mut Vec<u8>, keyword: &str, text: &str) {
        let mut data = Vec::with_capacity(keyword.len() + 1 + text.len());
        data.extend_from_slice(keyword.as_bytes());
        data.push(0); // Null separator
        data.extend_from_slice(text.as_bytes());
        Self::write_chunk(output, b"tEXt", &data);
    }
}

/// PNG metadata decoder extracting embedded provenance and header dimensions.
pub struct PngDecoder;

impl PngDecoder {
    /// Extracts all embedded text metadata and header dimensions from PNG bytes.
    pub fn extract_metadata(
        png_bytes: &[u8],
    ) -> Result<(u32, u32, HashMap<String, String>), ImageGenError> {
        if png_bytes.len() < 8 || png_bytes[0..8] != PNG_SIGNATURE {
            return Err(ImageGenError::PngError("Invalid PNG signature".to_string()));
        }

        let mut offset = 8;
        let len = png_bytes.len();
        let mut width = 0;
        let mut height = 0;
        let mut text_metadata = HashMap::new();

        while offset + 8 <= len {
            let chunk_len = u32::from_be_bytes([
                png_bytes[offset],
                png_bytes[offset + 1],
                png_bytes[offset + 2],
                png_bytes[offset + 3],
            ]) as usize;

            let chunk_type = &png_bytes[offset + 4..offset + 8];
            offset += 8;

            if offset + chunk_len + 4 > len {
                return Err(ImageGenError::PngError(
                    "Truncated PNG chunk payload".to_string(),
                ));
            }

            let chunk_data = &png_bytes[offset..offset + chunk_len];
            let expected_crc = u32::from_be_bytes([
                png_bytes[offset + chunk_len],
                png_bytes[offset + chunk_len + 1],
                png_bytes[offset + chunk_len + 2],
                png_bytes[offset + chunk_len + 3],
            ]);

            let actual_crc = calculate_crc32(&[chunk_type, chunk_data]);
            if actual_crc != expected_crc {
                return Err(ImageGenError::PngError(format!(
                    "CRC-32 checksum mismatch in {:?} chunk",
                    String::from_utf8_lossy(chunk_type)
                )));
            }

            if chunk_type == b"IHDR" {
                if chunk_len < 8 {
                    return Err(ImageGenError::PngError("Truncated IHDR chunk".to_string()));
                }
                width = u32::from_be_bytes([
                    chunk_data[0],
                    chunk_data[1],
                    chunk_data[2],
                    chunk_data[3],
                ]);
                height = u32::from_be_bytes([
                    chunk_data[4],
                    chunk_data[5],
                    chunk_data[6],
                    chunk_data[7],
                ]);
            } else if chunk_type == b"tEXt" {
                if let Some(null_pos) = chunk_data.iter().position(|&b| b == 0) {
                    let key = String::from_utf8_lossy(&chunk_data[0..null_pos]).to_string();
                    let val = String::from_utf8_lossy(&chunk_data[null_pos + 1..]).to_string();
                    text_metadata.insert(key, val);
                }
            } else if chunk_type == b"IEND" {
                break;
            }

            offset += chunk_len + 4; // skip data and CRC
        }

        Ok((width, height, text_metadata))
    }

    /// Recovers full `ImageProvenance` from PNG text metadata.
    pub fn extract_provenance(png_bytes: &[u8]) -> Result<ImageProvenance, ImageGenError> {
        let (width, height, meta) = Self::extract_metadata(png_bytes)?;

        let prompt = meta
            .get("miniai:prompt")
            .cloned()
            .unwrap_or_else(|| "Unknown prompt".to_string());
        let negative_prompt = meta.get("miniai:negative_prompt").cloned();
        let seed = meta
            .get("miniai:seed")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let steps = meta
            .get("miniai:steps")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let guidance_scale = meta
            .get("miniai:guidance_scale")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(7.5);
        let sha256 = meta.get("miniai:sha256").cloned().unwrap_or_default();
        let model_id = meta
            .get("miniai:model_id")
            .cloned()
            .unwrap_or_else(|| "unknown_model".to_string());
        let backend = meta
            .get("miniai:backend")
            .cloned()
            .unwrap_or_else(|| "cpu".to_string());
        let created_at_epoch_secs = meta
            .get("miniai:timestamp")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(ImageProvenance {
            prompt,
            negative_prompt,
            seed,
            steps,
            guidance_scale,
            width,
            height,
            sha256,
            model_id,
            backend,
            created_at_epoch_secs,
        })
    }
}
