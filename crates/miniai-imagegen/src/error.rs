use thiserror::Error;

/// Domain errors produced during image generation, latent diffusion, and PNG encoding.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ImageGenError {
    #[error("Invalid image dimensions ({width}x{height}): {details}")]
    InvalidDimension {
        width: u32,
        height: u32,
        details: String,
    },

    #[error("Invalid step count ({steps}): must be between {min} and {max}")]
    InvalidStepCount { steps: u32, min: u32, max: u32 },

    #[error("Invalid guidance scale ({scale}): must be between {min} and {max}")]
    InvalidGuidanceScale { scale: f32, min: f32, max: f32 },

    #[error("Invalid refinement strength ({strength}): must be between {min} and {max}")]
    InvalidStrength { strength: f32, min: f32, max: f32 },

    #[error("Input image dimension mismatch: expected {expected_w}x{expected_h}, got {actual_w}x{actual_h}")]
    InputImageDimensionMismatch {
        expected_w: u32,
        expected_h: u32,
        actual_w: u32,
        actual_h: u32,
    },

    #[error("Inpainting mask dimension mismatch: expected {expected_w}x{expected_h}, got {actual_w}x{actual_h}")]
    MaskDimensionMismatch {
        expected_w: u32,
        expected_h: u32,
        actual_w: u32,
        actual_h: u32,
    },

    #[error("Image generation was cooperatively cancelled by caller")]
    GenerationCancelled,

    #[error("Image generation model weights are not loaded")]
    ModelNotLoaded,

    #[error("PNG encoding or decoding error: {0}")]
    PngError(String),

    #[error("I/O error during image generation operation: {0}")]
    IoError(String),
}
