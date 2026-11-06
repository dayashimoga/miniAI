use serde::{Deserialize, Serialize};

use crate::error::ImageGenError;

/// Operational mode for image synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMode {
    /// Pure text-to-image synthesis from prompt and random latent noise.
    TextToImage,
    /// Image-to-image refinement conditioned on prompt and noise strength.
    ImageToImage,
    /// Masked inpainting replacing designated regions while preserving unmasked canvas.
    Inpainting,
}

/// Configuration parameters for image generation and editing tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageGenConfig {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub guidance_scale: f32,
    pub seed: u64,
    pub strength: f32,
    pub mode: GenerationMode,
}

impl Default for ImageGenConfig {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            negative_prompt: None,
            width: 512,
            height: 512,
            steps: 20,
            guidance_scale: 7.5,
            seed: 42,
            strength: 0.75,
            mode: GenerationMode::TextToImage,
        }
    }
}

impl ImageGenConfig {
    pub const MIN_DIMENSION: u32 = 128;
    pub const MAX_DIMENSION: u32 = 2048;
    pub const DIMENSION_STEP: u32 = 64;
    pub const MIN_STEPS: u32 = 1;
    pub const MAX_STEPS: u32 = 100;
    pub const MIN_GUIDANCE: f32 = 1.0;
    pub const MAX_GUIDANCE: f32 = 25.0;
    pub const MIN_STRENGTH: f32 = 0.05;
    pub const MAX_STRENGTH: f32 = 1.0;

    /// Validates all parameters, ensuring multiples of 64 and bounded values.
    pub fn validate(&self) -> Result<(), ImageGenError> {
        if self.prompt.trim().is_empty() {
            return Err(ImageGenError::InvalidDimension {
                width: self.width,
                height: self.height,
                details: "Prompt cannot be empty".to_string(),
            });
        }

        if self.width < Self::MIN_DIMENSION
            || self.width > Self::MAX_DIMENSION
            || !self.width.is_multiple_of(Self::DIMENSION_STEP)
        {
            return Err(ImageGenError::InvalidDimension {
                width: self.width,
                height: self.height,
                details: format!(
                    "Width ({}) must be a multiple of {} between {} and {}",
                    self.width,
                    Self::DIMENSION_STEP,
                    Self::MIN_DIMENSION,
                    Self::MAX_DIMENSION
                ),
            });
        }

        if self.height < Self::MIN_DIMENSION
            || self.height > Self::MAX_DIMENSION
            || !self.height.is_multiple_of(Self::DIMENSION_STEP)
        {
            return Err(ImageGenError::InvalidDimension {
                width: self.width,
                height: self.height,
                details: format!(
                    "Height ({}) must be a multiple of {} between {} and {}",
                    self.height,
                    Self::DIMENSION_STEP,
                    Self::MIN_DIMENSION,
                    Self::MAX_DIMENSION
                ),
            });
        }

        if self.steps < Self::MIN_STEPS || self.steps > Self::MAX_STEPS {
            return Err(ImageGenError::InvalidStepCount {
                steps: self.steps,
                min: Self::MIN_STEPS,
                max: Self::MAX_STEPS,
            });
        }

        if self.guidance_scale < Self::MIN_GUIDANCE || self.guidance_scale > Self::MAX_GUIDANCE {
            return Err(ImageGenError::InvalidGuidanceScale {
                scale: self.guidance_scale,
                min: Self::MIN_GUIDANCE,
                max: Self::MAX_GUIDANCE,
            });
        }

        if self.mode != GenerationMode::TextToImage
            && (self.strength < Self::MIN_STRENGTH || self.strength > Self::MAX_STRENGTH)
        {
            return Err(ImageGenError::InvalidStrength {
                strength: self.strength,
                min: Self::MIN_STRENGTH,
                max: Self::MAX_STRENGTH,
            });
        }

        Ok(())
    }
}

/// Telemetry payload emitted after each denoising step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepProgress {
    pub step: u32,
    pub total_steps: u32,
    pub progress_percent: f32,
    pub current_noise_level: f32,
    pub elapsed_ms: u64,
}

/// Cryptographic and contextual provenance metadata embedded into the output PNG.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageProvenance {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub seed: u64,
    pub steps: u32,
    pub guidance_scale: f32,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub model_id: String,
    pub backend: String,
    pub created_at_epoch_secs: u64,
}

/// Final output of an image generation or editing pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageGenResult {
    /// Lossless encoded PNG byte stream with embedded provenance chunks.
    pub png_bytes: Vec<u8>,
    /// Raw uncompressed interleaved RGB pixel bytes.
    pub raw_rgb: Vec<u8>,
    /// Provenance metadata matching the embedded chunk.
    pub provenance: ImageProvenance,
    /// Total wall-clock generation duration in milliseconds.
    pub total_duration_ms: u64,
}
