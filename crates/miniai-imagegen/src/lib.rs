#![forbid(unsafe_code)]

pub mod adapter;
pub mod engine;
pub mod error;
pub mod png;
pub mod types;

pub use adapter::ImageGenAdapter;
pub use engine::ImageGenEngine;
pub use error::ImageGenError;
pub use png::{calculate_adler32, calculate_crc32, PngDecoder, PngEncoder};
pub use types::{GenerationMode, ImageGenConfig, ImageGenResult, ImageProvenance, StepProgress};
