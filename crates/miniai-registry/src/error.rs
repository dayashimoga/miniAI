use thiserror::Error;

/// Errors produced by the model registry and download subsystem.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("I/O error: {message}")]
    IoError { message: String },

    #[error("Storage error: {0}")]
    StorageError(#[from] miniai_storage::error::StorageError),

    #[error("Core error: {0}")]
    CoreError(#[from] miniai_core::MiniAiError),

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Model '{model_id}' not found in registry")]
    ModelNotFound { model_id: String },

    #[error("SHA-256 verification failed for model '{model_id}': expected {expected}, computed {actual}")]
    Sha256Mismatch {
        model_id: String,
        expected: String,
        actual: String,
    },

    #[error("Download error for model '{model_id}': {message}")]
    DownloadError { model_id: String, message: String },

    #[error(
        "Local-Only Privacy Mode prohibited remote download of model '{model_id}' from '{url}'"
    )]
    PrivacyModeBlocked { model_id: String, url: String },

    #[error("HTTP range request rejected with status code {status}")]
    HttpError { status: u16, message: String },

    #[error("Corrupted staging file for model '{model_id}': {message}")]
    CorruptedStagingFile { model_id: String, message: String },
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            message: err.to_string(),
        }
    }
}
