use thiserror::Error;

/// Specific error types for speech recognition and audio processing.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum SpeechAsrError {
    #[error("Invalid or unrecognized audio format: {0}")]
    InvalidFormat(String),

    #[error("Audio file size {size} bytes exceeds maximum limit of {limit} bytes")]
    FileTooLarge { size: usize, limit: usize },

    #[error("Audio duration {duration_secs:.2}s exceeds maximum limit of {limit_secs:.2}s")]
    AudioTooLong { duration_secs: f32, limit_secs: f32 },

    #[error("Unsupported audio sample rate: {rate} Hz (must be 8000 Hz to 192000 Hz)")]
    UnsupportedSampleRate { rate: u32 },

    #[error("Unsupported audio channels count: {channels} (maximum 8 supported)")]
    UnsupportedChannels { channels: u16 },

    #[error("Unsupported audio bit depth: {depth}-bit (must be 8, 16, 24, or 32-bit)")]
    UnsupportedBitDepth { depth: u16 },

    #[error("Corrupted or malformed audio payload: {0}")]
    CorruptPayload(String),

    #[error("Speech recognition model weights are not loaded")]
    UnloadedModel,

    #[error("I/O error during audio processing: {0}")]
    Io(String),

    #[error("Audio processing failed: {0}")]
    Processing(String),
}
