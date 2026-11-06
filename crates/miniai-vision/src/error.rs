use thiserror::Error;

/// Errors produced during image sanitization, tensor preprocessing, and vision inference.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum VisionError {
    #[error("Invalid or unrecognized image magic bytes: {detected}")]
    InvalidMagicBytes { detected: String },

    #[error("Unsupported image format: {extension}")]
    UnsupportedFormat { extension: String },

    #[error(
        "Image file size ({size_bytes} bytes) exceeds maximum allowable limit ({max_bytes} bytes)"
    )]
    FileTooLarge { size_bytes: usize, max_bytes: usize },

    #[error(
        "Image dimensions ({width}x{height}) exceed maximum allowable limit ({max_dim}x{max_dim})"
    )]
    DimensionsExceeded {
        width: u32,
        height: u32,
        max_dim: u32,
    },

    #[error("Potential decompression bomb rejected: total pixels ({total_pixels}) exceed ceiling ({max_pixels})")]
    DecompressionBomb { total_pixels: u64, max_pixels: u64 },

    #[error("Corrupt or truncated image payload: {details}")]
    CorruptPayload { details: String },

    #[error("Image preprocessing or tensor transformation failed: {details}")]
    PreprocessingFailed { details: String },

    #[error("Vision model weights are not loaded")]
    ModelNotLoaded,

    #[error("Vision inference failed: {details}")]
    InferenceFailed { details: String },

    #[error("I/O error during vision operation: {0}")]
    IoError(String),
}
