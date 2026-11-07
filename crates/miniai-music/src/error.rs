use thiserror::Error;

/// Errors arising during polyphonic music synthesis and adapter operations.
#[derive(Debug, Error)]
pub enum MusicError {
    #[error("Empty music prompt provided")]
    PromptEmpty,

    #[error("Duration {duration}s is out of allowed bounds [{min}s, {max}s]")]
    DurationOutOfBounds { duration: f32, min: f32, max: f32 },

    #[error("Tempo {bpm} BPM is out of allowed bounds [{min}, {max}]")]
    TempoOutOfBounds { bpm: u16, min: u16, max: u16 },

    #[error("Invalid synthesis parameter: {0}")]
    InvalidParameter(String),

    #[error("Model is not loaded")]
    UnloadedModel,

    #[error("Music synthesis was cancelled")]
    Cancelled,

    #[error("Audio encoding error: {0}")]
    EncodingError(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Corrupt payload or checksum mismatch: {0}")]
    CorruptPayload(String),

    #[error("Music synthesis processing error: {0}")]
    Processing(String),
}
