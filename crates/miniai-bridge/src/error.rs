use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Storage error: {0}")]
    Storage(#[from] miniai_storage::StorageError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Admission guard rejected workload: {0}")]
    AdmissionFailed(String),

    #[error("Unknown bridge action: {0}")]
    UnknownAction(String),

    #[error("Null pointer provided to FFI function")]
    NullPointer,

    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}
