use serde::{Deserialize, Serialize};

/// Supported audio container formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    Wav,
    Flac,
    Ogg,
}

impl AudioFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Flac => "flac",
            AudioFormat::Ogg => "ogg",
        }
    }
}

/// Metadata extracted from sanitized audio file headers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub duration_secs: f32,
    pub sample_count: usize,
    pub sha256: String,
}

/// Standardized float32 audio sample buffer (16 kHz mono).
#[derive(Debug, Clone, PartialEq)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioBuffer {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            0.0
        } else {
            self.samples.len() as f32 / (self.sample_rate as f32 * self.channels as f32)
        }
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn slice_seconds(&self, start_secs: f32, end_secs: f32) -> &[f32] {
        let start_idx =
            ((start_secs * self.sample_rate as f32).round() as usize).min(self.samples.len());
        let end_idx = ((end_secs * self.sample_rate as f32).round() as usize)
            .min(self.samples.len())
            .max(start_idx);
        &self.samples[start_idx..end_idx]
    }
}

/// Speech segment representing a timestamped utterance or phrase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub id: usize,
    pub start_secs: f32,
    pub end_secs: f32,
    pub text: String,
    pub confidence: f32,
    pub tokens: Vec<String>,
}

/// Voice Activity Detection frame metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VadFrame {
    pub start_secs: f32,
    pub end_secs: f32,
    pub energy: f32,
    pub zero_crossing_rate: f32,
    pub is_speech: bool,
}

/// Complete ASR transcription result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AsrTranscriptionResult {
    pub audio_sha256: String,
    pub duration_secs: f32,
    pub language: String,
    pub full_text: String,
    pub segments: Vec<SpeechSegment>,
    pub vad_segments: Vec<SpeechSegment>,
    pub confidence: f32,
    pub model_id: String,
}
