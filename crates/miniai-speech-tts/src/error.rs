use thiserror::Error;

/// Errors arising during Text-to-Speech synthesis and offline voice conversation loops.
#[derive(Debug, Error)]
pub enum SpeechTtsError {
    #[error("Text exceeds maximum allowed chunk length ({length} > {max})")]
    TextTooLong { length: usize, max: usize },

    #[error("Empty text provided for synthesis")]
    EmptyText,

    #[error("Invalid synthesis parameter: {0}")]
    InvalidParameter(String),

    #[error("Audio encoding error: {0}")]
    EncodingError(String),

    #[error("Model is not loaded")]
    UnloadedModel,

    #[error("Synthesis was cancelled")]
    Cancelled,

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Corrupt payload or checksum mismatch: {0}")]
    CorruptPayload(String),

    #[error("Speech ASR error during voice loop: {0}")]
    AsrError(String),

    #[error("LLM error during voice loop: {0}")]
    LlmError(String),

    #[error("Synthesis processing error: {0}")]
    Processing(String),
}

