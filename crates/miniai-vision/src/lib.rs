#![forbid(unsafe_code)]

pub mod adapter;
pub mod engine;
pub mod error;
pub mod sanitizer;
pub mod tensor;
pub mod types;

pub use adapter::VisionAdapter;
pub use engine::VisionEngine;
pub use error::VisionError;
pub use sanitizer::{
    ImageSanitizer, MAX_IMAGE_DIMENSION, MAX_IMAGE_FILE_SIZE_BYTES, MAX_IMAGE_TOTAL_PIXELS,
};
pub use tensor::{ImageTensor, DEFAULT_VISION_RESOLUTION, IMAGENET_MEAN, IMAGENET_STD};
pub use types::{
    BoundingBox, ImageDimensions, ImageFormat, ImageMetadata, OcrResult, OcrTextElement,
    VisualDescription, VisualQaAnswer, VisualQaQuery,
};
