use serde::{Deserialize, Serialize};

use crate::error::SpeechTtsError;

/// Voice profiles modeling distinct vocal characteristics, timbre, and formants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceProfile {
    /// Neutral, versatile, balanced voice (~160 Hz base f0).
    #[default]
    Alloy,
    /// Warm, resonant baritone tone (~130 Hz base f0).
    Echo,
    /// Expressive, lively narrative voice (~180 Hz base f0).
    Fable,
    /// Authoritative, deep baritone (~105 Hz base f0).
    Onyx,
    /// Bright, energetic, dynamic feminine voice (~220 Hz base f0).
    Nova,
    /// Soft, clear, melodic voice (~250 Hz base f0).
    Shimmer,
    /// Custom voice profile with user-defined fundamental and formant scale.
    Custom {
        name: String,
        base_f0_hz: f32,
        formant_scale: f32,
    },
}

impl VoiceProfile {
    /// Returns all standard preset profiles.
    pub fn presets() -> [VoiceProfile; 6] {
        [
            Self::Alloy,
            Self::Echo,
            Self::Fable,
            Self::Onyx,
            Self::Nova,
            Self::Shimmer,
        ]
    }

    /// Display name of the voice profile.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Alloy => "Alloy (Neutral & Balanced)",
            Self::Echo => "Echo (Warm & Resonant)",
            Self::Fable => "Fable (Expressive Narrative)",
            Self::Onyx => "Onyx (Deep Baritone)",
            Self::Nova => "Nova (Bright & Dynamic)",
            Self::Shimmer => "Shimmer (Clear & Melodic)",
            Self::Custom { name, .. } => name.as_str(),
        }
    }

    /// Fundamental vocal tract frequency $f_0$ in Hertz.
    pub fn base_frequency_hz(&self) -> f32 {
        match self {
            Self::Alloy => 160.0,
            Self::Echo => 130.0,
            Self::Fable => 180.0,
            Self::Onyx => 105.0,
            Self::Nova => 220.0,
            Self::Shimmer => 250.0,
            Self::Custom { base_f0_hz, .. } => *base_f0_hz,
        }
    }

    /// Formant frequency scaling multiplier relative to neutral standard.
    pub fn formant_scale(&self) -> f32 {
        match self {
            Self::Alloy => 1.00,
            Self::Echo => 0.92,
            Self::Fable => 1.05,
            Self::Onyx => 0.85,
            Self::Nova => 1.15,
            Self::Shimmer => 1.22,
            Self::Custom { formant_scale, .. } => *formant_scale,
        }
    }

    /// Overtone harmonic warmth decay factor.
    pub fn harmonic_decay(&self) -> f32 {
        match self {
            Self::Alloy => 0.65,
            Self::Echo => 0.75,
            Self::Fable => 0.60,
            Self::Onyx => 0.80,
            Self::Nova => 0.55,
            Self::Shimmer => 0.50,
            Self::Custom { .. } => 0.65,
        }
    }
}

/// Synthesis configuration parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtsSynthesisConfig {
    /// Voice profile selection.
    pub voice: VoiceProfile,
    /// Audio sample rate in Hz (22050 or 24000).
    pub sample_rate: u32,
    /// Speaking rate multiplier (0.5 to 2.0, default 1.0).
    pub speed: f32,
    /// Pitch shift in semitones (-12.0 to +12.0, default 0.0).
    pub pitch_semitones: f32,
    /// Output gain / volume multiplier (0.0 to 2.0, default 1.0).
    pub volume_gain: f32,
    /// Optional seed for deterministic synthesis.
    pub seed: Option<u64>,
}

impl Default for TtsSynthesisConfig {
    fn default() -> Self {
        Self {
            voice: VoiceProfile::Alloy,
            sample_rate: 22050,
            speed: 1.0,
            pitch_semitones: 0.0,
            volume_gain: 1.0,
            seed: Some(42),
        }
    }
}

impl TtsSynthesisConfig {
    /// Validates parameter ranges.
    pub fn validate(&self) -> Result<(), SpeechTtsError> {
        if self.sample_rate != 22050 && self.sample_rate != 24000 {
            return Err(SpeechTtsError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz (allowed: 22050, 24000)",
                self.sample_rate
            )));
        }
        if self.speed < 0.5 || self.speed > 2.0 {
            return Err(SpeechTtsError::InvalidParameter(format!(
                "Speed must be between 0.5 and 2.0, got {}",
                self.speed
            )));
        }
        if self.pitch_semitones < -12.0 || self.pitch_semitones > 12.0 {
            return Err(SpeechTtsError::InvalidParameter(format!(
                "Pitch semitones must be between -12.0 and +12.0, got {}",
                self.pitch_semitones
            )));
        }
        if self.volume_gain < 0.0 || self.volume_gain > 2.0 {
            return Err(SpeechTtsError::InvalidParameter(format!(
                "Volume gain must be between 0.0 and 2.0, got {}",
                self.volume_gain
            )));
        }
        if let VoiceProfile::Custom {
            base_f0_hz,
            formant_scale,
            ..
        } = &self.voice
        {
            if *base_f0_hz < 50.0 || *base_f0_hz > 500.0 {
                return Err(SpeechTtsError::InvalidParameter(format!(
                    "Custom base f0 must be between 50 Hz and 500 Hz, got {}",
                    base_f0_hz
                )));
            }
            if *formant_scale < 0.5 || *formant_scale > 2.0 {
                return Err(SpeechTtsError::InvalidParameter(format!(
                    "Custom formant scale must be between 0.5 and 2.0, got {}",
                    formant_scale
                )));
            }
        }
        Ok(())
    }
}

/// Synthesized audio output payload with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtsAudioOutput {
    /// Valid encoded RIFF/WAVE container bytes.
    pub wav_bytes: Vec<u8>,
    /// Raw mono float32 PCM samples in [-1.0, 1.0].
    pub pcm_samples: Vec<f32>,
    /// Total duration in seconds.
    pub duration_secs: f32,
    /// Audio sample rate in Hz.
    pub sample_rate: u32,
    /// Channel count (always 1 for mono).
    pub channels: u16,
    /// Cryptographic SHA-256 hex digest of the output WAV bytes.
    pub sha256: String,
    /// Number of phonetic units synthesized.
    pub phoneme_count: usize,
}

/// Synthesis progress event for telemetry callbacks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtsProgress {
    /// Current synthesis stage name.
    pub stage: String,
    /// 0-indexed current text chunk being processed.
    pub current_chunk: usize,
    /// Total text chunks.
    pub total_chunks: usize,
    /// Overall completion ratio in [0.0, 1.0].
    pub progress_ratio: f32,
}

/// Latency and timing breakdown of an offline voice conversation turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceTurnMetrics {
    /// Latency of speech ASR transcription in milliseconds.
    pub asr_latency_ms: f64,
    /// Time-to-first-token of LLM response generation in milliseconds.
    pub llm_ttft_ms: f64,
    /// Total execution time of LLM response generation in milliseconds.
    pub llm_total_latency_ms: f64,
    /// Latency of TTS speech synthesis in milliseconds.
    pub tts_latency_ms: f64,
    /// Total end-to-end roundtrip turnaround time in milliseconds.
    pub total_turnaround_ms: f64,
}

/// Complete result of a single offline voice conversation turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceTurnResult {
    /// Transcribed user query from audio input.
    pub user_transcription: String,
    /// Generated assistant response text from LLM.
    pub agent_response_text: String,
    /// Synthesized assistant speech audio output.
    pub synthesized_audio: TtsAudioOutput,
    /// Turnaround latency telemetry metrics.
    pub metrics: VoiceTurnMetrics,
}
