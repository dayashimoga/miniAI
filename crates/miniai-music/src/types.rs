use serde::{Deserialize, Serialize};

use crate::error::MusicError;

pub const MIN_DURATION_SECS: f32 = 1.0;
pub const MAX_DURATION_SECS: f32 = 120.0;
pub const MIN_TEMPO_BPM: u16 = 40;
pub const MAX_TEMPO_BPM: u16 = 240;

/// Musical scale and harmonic key representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum MusicalScale {
    #[default]
    CMajor,
    AMinor,
    GMajor,
    EMinor,
    DDorian,
}

impl MusicalScale {
    /// Parses musical scale from string name (case-insensitive).
    pub fn from_name(name: &str) -> Self {
        let norm = name.to_lowercase();
        if norm.contains("a min") || norm.contains("a_min") || norm == "am" {
            Self::AMinor
        } else if norm.contains("g maj") || norm.contains("g_maj") || norm == "g" {
            Self::GMajor
        } else if norm.contains("e min") || norm.contains("e_min") || norm == "em" {
            Self::EMinor
        } else if norm.contains("dorian") || norm.contains("d dor") {
            Self::DDorian
        } else {
            Self::CMajor
        }
    }

    pub fn scale_name(&self) -> &'static str {
        match self {
            Self::CMajor => "C Major",
            Self::AMinor => "A Minor",
            Self::GMajor => "G Major",
            Self::EMinor => "E Minor",
            Self::DDorian => "D Dorian",
        }
    }

    /// Frequencies for scale degrees in octave 4.
    pub fn frequencies(&self) -> &'static [f32] {
        match self {
            Self::CMajor => &[
                261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25,
            ],
            Self::AMinor => &[
                220.00, 246.94, 261.63, 293.66, 329.63, 349.23, 392.00, 440.00,
            ],
            Self::GMajor => &[
                196.00, 220.00, 246.94, 261.63, 293.66, 329.63, 369.99, 392.00,
            ],
            Self::EMinor => &[
                164.81, 185.00, 196.00, 220.00, 246.94, 261.63, 293.66, 329.63,
            ],
            Self::DDorian => &[
                146.83, 164.81, 174.61, 196.00, 220.00, 246.94, 261.63, 293.66,
            ],
        }
    }

    /// Root fundamental frequency in Hz (bass octave).
    pub fn root_bass_frequency(&self) -> f32 {
        self.frequencies()[0] * 0.5
    }

    /// Returns the triadic chord frequencies (root, third, fifth) for a given scale degree (0-indexed).
    pub fn chord_triad(&self, degree: usize) -> [f32; 3] {
        let f = self.frequencies();
        let len = f.len();
        let root = f[degree % len];
        let third = f[(degree + 2) % len];
        let fifth = f[(degree + 4) % len];
        [root, third, fifth]
    }
}

/// Configuration parameters for music generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicGenConfig {
    /// Style, mood, or instrumentation prompt (e.g., "Uplifting synthwave in C Major", "Lo-fi ambient piano").
    pub prompt: String,
    /// Duration of synthesized track in seconds (1.0 to 120.0, default 10.0).
    pub duration_secs: f32,
    /// Tempo in Beats Per Minute (40 to 240, default 120).
    pub tempo_bpm: u16,
    /// Audio sample rate in Hz (22050, 44100, 48000, default 44100).
    pub sample_rate: u32,
    /// Audio channel count (1 = mono, 2 = stereo soundstage, default 2).
    pub channels: u16,
    /// Harmonic scale and key.
    pub scale_key: String,
    /// Optional seed for deterministic synthesis.
    pub seed: Option<u64>,
}

impl Default for MusicGenConfig {
    fn default() -> Self {
        Self {
            prompt: "Ambient piano and warm strings".to_string(),
            duration_secs: 10.0,
            tempo_bpm: 120,
            sample_rate: 44100,
            channels: 2,
            scale_key: "C Major".to_string(),
            seed: Some(42),
        }
    }
}

impl MusicGenConfig {
    /// Validates parameter ranges against multimodal safety bounds.
    pub fn validate(&self) -> Result<(), MusicError> {
        if self.prompt.trim().is_empty() {
            return Err(MusicError::PromptEmpty);
        }
        if self.duration_secs < MIN_DURATION_SECS || self.duration_secs > MAX_DURATION_SECS {
            return Err(MusicError::DurationOutOfBounds {
                duration: self.duration_secs,
                min: MIN_DURATION_SECS,
                max: MAX_DURATION_SECS,
            });
        }
        if self.tempo_bpm < MIN_TEMPO_BPM || self.tempo_bpm > MAX_TEMPO_BPM {
            return Err(MusicError::TempoOutOfBounds {
                bpm: self.tempo_bpm,
                min: MIN_TEMPO_BPM,
                max: MAX_TEMPO_BPM,
            });
        }
        if self.sample_rate != 22050 && self.sample_rate != 44100 && self.sample_rate != 48000 {
            return Err(MusicError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz (allowed: 22050, 44100, 48000)",
                self.sample_rate
            )));
        }
        if self.channels != 1 && self.channels != 2 {
            return Err(MusicError::InvalidParameter(format!(
                "Channels must be 1 (mono) or 2 (stereo), got {}",
                self.channels
            )));
        }
        Ok(())
    }
}

/// Output metadata and audio container from music generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicTrackOutput {
    /// Valid standard RIFF/WAVE container bytes (16-bit PCM).
    pub wav_bytes: Vec<u8>,
    /// Duration of audio in seconds.
    pub duration_secs: f32,
    /// Audio sample rate in Hz.
    pub sample_rate: u32,
    /// Channel count (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Tempo in BPM.
    pub tempo_bpm: u16,
    /// Harmonic scale key name.
    pub scale_key: String,
    /// Cryptographic SHA-256 hex digest of the output WAV file.
    pub sha256: String,
    /// Number of audio samples per channel.
    pub num_samples_per_channel: usize,
}

/// Generation progress event for UI telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicProgress {
    /// Current synthesis stage description.
    pub stage: String,
    /// Current musical measure being rendered.
    pub current_measure: usize,
    /// Total musical measures.
    pub total_measures: usize,
    /// Completion ratio in [0.0, 1.0].
    pub progress_ratio: f32,
}
