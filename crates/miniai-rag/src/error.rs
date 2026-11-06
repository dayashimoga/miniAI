use std::path::PathBuf;
use thiserror::Error;

/// RAG subsystem errors for document indexing, chunking, and retrieval.
#[derive(Debug, Error)]
pub enum RagError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path traversal rejected: '{0}' attempts to escape workspace root")]
    PathTraversal(String),

    #[error("Access outside workspace rejected: path '{path}' is not within root '{root}'")]
    PathOutsideWorkspace { path: PathBuf, root: PathBuf },

    #[error("Sensitive file rejected: '{0}' matched security exclusion pattern")]
    SensitiveFileExcluded(String),

    #[error("Binary file rejected: '{0}' contains non-text or null bytes")]
    BinaryFileRejected(String),

    #[error(
        "File exceeds maximum allowed size ({actual_bytes} bytes > {max_bytes} bytes): {path}"
    )]
    FileTooLarge {
        path: String,
        actual_bytes: usize,
        max_bytes: usize,
    },

    #[error("Document is empty: {0}")]
    EmptyDocument(String),

    #[error("UTF-8 decoding error for file '{path}': {source}")]
    InvalidEncoding {
        path: String,
        source: std::string::FromUtf8Error,
    },

    #[error("Serialization error: {0}")]
    Serialization(String),
}
