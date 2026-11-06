use serde::{Deserialize, Serialize};

/// Supported image formats for vision understanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
}

impl ImageFormat {
    /// Returns the MIME type associated with the image format.
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
        }
    }

    /// Returns standard file extension for the format.
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
        }
    }
}

/// 2D dimensions of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Total pixel count.
    pub const fn total_pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Aspect ratio as width / height.
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Human-readable aspect ratio label.
    pub fn aspect_ratio_label(&self) -> &'static str {
        let ar = self.aspect_ratio();
        if (ar - 1.0).abs() < 0.05 {
            "Square (1:1)"
        } else if ar > 1.05 {
            "Landscape"
        } else {
            "Portrait"
        }
    }
}

/// Validated image metadata after header sanitization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub format: ImageFormat,
    pub dimensions: ImageDimensions,
    pub size_bytes: usize,
    pub sha256: String,
    pub has_alpha: bool,
}

/// Normalized bounding box coordinates within [0.0, 1.0] relative to image dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub confidence: f32,
    pub label: String,
}

impl BoundingBox {
    pub fn new(
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        confidence: f32,
        label: impl Into<String>,
    ) -> Self {
        Self {
            x_min: x_min.clamp(0.0, 1.0),
            y_min: y_min.clamp(0.0, 1.0),
            x_max: x_max.clamp(0.0, 1.0),
            y_max: y_max.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            label: label.into(),
        }
    }
}

/// A single detected OCR text element with localized bounding box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrTextElement {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub line_number: u32,
}

/// Complete OCR extraction output for an image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrResult {
    pub full_text: String,
    pub elements: Vec<OcrTextElement>,
    pub word_count: usize,
    pub processing_time_ms: u64,
}

/// Visual scene description and high-level comprehension output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualDescription {
    pub caption: String,
    pub detected_tags: Vec<String>,
    pub dominant_colors: Vec<String>,
    pub aspect_ratio_label: String,
    pub estimated_complexity: f32,
    pub processing_time_ms: u64,
}

/// Natural language question posed about an image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualQaQuery {
    pub prompt: String,
}

/// Grounded response to a Visual QA prompt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualQaAnswer {
    pub prompt: String,
    pub answer: String,
    pub grounding_evidence: Vec<String>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}
