use crate::error::SpeechTtsError;

/// Lossless standard RIFF/WAVE audio encoder.
pub struct WavEncoder;

impl WavEncoder {
    /// Encodes mono float32 samples in [-1.0, 1.0] to a 16-bit signed PCM WAV container.
    pub fn encode_pcm16_mono(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, SpeechTtsError> {
        if sample_rate == 0 {
            return Err(SpeechTtsError::InvalidParameter(
                "Sample rate cannot be zero".to_string(),
            ));
        }

        let num_channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let bytes_per_sample: u32 = 2;
        let block_align: u16 = num_channels * (bits_per_sample / 8);
        let byte_rate: u32 = sample_rate * (block_align as u32);
        let data_len: u32 = (samples.len() as u32)
            .checked_mul(bytes_per_sample)
            .ok_or_else(|| SpeechTtsError::EncodingError("Audio payload too large".to_string()))?;
        let riff_chunk_size: u32 = 36u32
            .checked_add(data_len)
            .ok_or_else(|| SpeechTtsError::EncodingError("RIFF chunk size overflow".to_string()))?;

        let mut out = Vec::with_capacity(44 + data_len as usize);

        // RIFF header
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&riff_chunk_size.to_le_bytes());
        out.extend_from_slice(b"WAVE");

        // "fmt " chunk
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size (16 for PCM)
        out.extend_from_slice(&1u16.to_le_bytes()); // audio format 1 = PCM
        out.extend_from_slice(&num_channels.to_le_bytes());
        out.extend_from_slice(&sample_rate.to_le_bytes());
        out.extend_from_slice(&byte_rate.to_le_bytes());
        out.extend_from_slice(&block_align.to_le_bytes());
        out.extend_from_slice(&bits_per_sample.to_le_bytes());

        // "data" chunk
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());

        // PCM 16-bit samples
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let s16 = if clamped >= 0.0 {
                (clamped * 32767.0).round() as i16
            } else {
                (clamped * 32768.0).round() as i16
            };
            out.extend_from_slice(&s16.to_le_bytes());
        }

        Ok(out)
    }

    /// Validates and parses header information from a RIFF/WAVE byte stream.
    pub fn parse_header(bytes: &[u8]) -> Result<(u32, u16, u16, usize), SpeechTtsError> {
        if bytes.len() < 44 {
            return Err(SpeechTtsError::EncodingError(
                "WAV file too short (< 44 bytes)".to_string(),
            ));
        }

        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(SpeechTtsError::EncodingError(
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

