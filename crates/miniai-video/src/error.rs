use thiserror::Error;

/// All error variants produced by the `miniai-video` crate.
#[derive(Debug, Error)]
pub enum VideoError {
    #[error("Invalid video container: {0}")]
    InvalidContainer(String),

    #[error("Corrupt or truncated video payload: {0}")]
    CorruptPayload(String),

    #[error("Video payload size ({size_bytes} bytes) exceeds safety limit ({limit_bytes} bytes)")]
    FileTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },

    #[error("Video duration ({duration_secs:.2}s) exceeds safety limit ({limit_secs:.2}s)")]
    DurationExceeded { duration_secs: f64, limit_secs: f64 },

    #[error("Video resolution ({width}x{height}) exceeds maximum dimension ({max_dim})")]
    ResolutionExceeded {
        width: u32,
        height: u32,
        max_dim: u32,
    },

    #[error("Invalid frame rate ({fps} fps). Must be between 1.0 and 60.0 fps")]
    InvalidFrameRate { fps: f64 },

    #[error("Frame buffer pool exhausted: requested {requested_bytes} bytes, only {available_bytes} available (budget: {budget_bytes} bytes)")]
    BufferPoolExhausted {
        requested_bytes: usize,
        available_bytes: usize,
        budget_bytes: usize,
    },

    #[error("Frame extraction error: {0}")]
    FrameExtractionError(String),

    #[error("Audio demux error: {0}")]
    AudioDemuxError(String),

    #[error("Model is not loaded")]
    ModelNotLoaded,

    #[error("Weights file not found at: {0}")]
    WeightsNotFound(String),

    #[error("Weights integrity mismatch: expected SHA-256 {expected}, computed {actual}")]
    WeightsIntegrityMismatch { expected: String, actual: String },

    #[error("Weights file corrupted or invalid JSON: {0}")]
    WeightsCorrupted(String),

    #[error("Video inference error: {0}")]
    InferenceError(String),

    #[error("Operation cancelled by user or timeout")]
    Cancelled,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
