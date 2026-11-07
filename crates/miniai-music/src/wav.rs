use crate::error::MusicError;

/// Standard RIFF/WAVE 16-bit PCM encoder for mono and stereo audio.
pub struct WavEncoder;

impl WavEncoder {
    /// Encodes interleaved float32 samples in [-1.0, 1.0] to a 16-bit signed PCM WAV container.
    pub fn encode_pcm16(
        samples: &[f32],
        channels: u16,
        sample_rate: u32,
    ) -> Result<Vec<u8>, MusicError> {
        if sample_rate == 0 {
            return Err(MusicError::InvalidParameter(
                "Sample rate cannot be zero".to_string(),
            ));
        }
        if channels != 1 && channels != 2 {
            return Err(MusicError::InvalidParameter(
                "Channels must be 1 or 2".to_string(),
            ));
        }

        let bits_per_sample: u16 = 16;
        let bytes_per_sample: u32 = 2;
        let block_align: u16 = channels * (bits_per_sample / 8);
        let byte_rate: u32 = sample_rate * (block_align as u32);
        let data_len: u32 = (samples.len() as u32)
            .checked_mul(bytes_per_sample)
            .ok_or_else(|| MusicError::EncodingError("Audio payload too large".to_string()))?;
        let riff_chunk_size: u32 = 36u32
            .checked_add(data_len)
            .ok_or_else(|| MusicError::EncodingError("RIFF chunk size overflow".to_string()))?;

        let mut out = Vec::with_capacity(44 + data_len as usize);

        // RIFF header
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&riff_chunk_size.to_le_bytes());
        out.extend_from_slice(b"WAVE");

        // "fmt " subchunk
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&sample_rate.to_le_bytes());
        out.extend_from_slice(&byte_rate.to_le_bytes());
        out.extend_from_slice(&block_align.to_le_bytes());
        out.extend_from_slice(&bits_per_sample.to_le_bytes());

        // "data" subchunk
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());

        // Clamped 16-bit PCM samples
        for &s in samples {
            let clamped = s.clamp(-1.0, 1.0);
            let s16 = if clamped >= 0.0 {
                (clamped * 32767.0).round() as i16
            } else {
                (clamped * 32768.0).round() as i16
            };
            out.extend_from_slice(&s16.to_le_bytes());
        }

        Ok(out)
    }

    /// Parses header information from a RIFF/WAVE byte stream.
    pub fn parse_header(bytes: &[u8]) -> Result<(u32, u16, u16, usize), MusicError> {
        if bytes.len() < 44 {
            return Err(MusicError::EncodingError(
                "WAV file too short (< 44 bytes)".to_string(),
            ));
        }

        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(MusicError::EncodingError(
                "Invalid RIFF/WAVE header magic bytes".to_string(),
            ));
        }

        let num_channels = u16::from_le_bytes([bytes[22], bytes[23]]);
        let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
        let bits_per_sample = u16::from_le_bytes([bytes[34], bytes[35]]);
        let data_len = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;

        Ok((sample_rate, num_channels, bits_per_sample, data_len))
    }
}

