use std::path::Path;
use std::sync::{Arc, RwLock};

use miniai_core::{
    AcceleratorBackend, AdapterHealth, MiniAiError, Modality, ModalityAdapter, RuntimeCapability,
};
use sha2::{Digest, Sha256};

use crate::engine::VisionEngine;
use crate::error::VisionError;
use crate::sanitizer::ImageSanitizer;
use crate::tensor::{ImageTensor, DEFAULT_VISION_RESOLUTION};
use crate::types::{ImageMetadata, OcrResult, VisualDescription, VisualQaAnswer, VisualQaQuery};

/// Pluggable, decoupled Vision Understanding runtime adapter implementing `ModalityAdapter`.
pub struct VisionAdapter {
    adapter_id: String,
    engine: Arc<RwLock<Option<VisionEngine>>>,
    active_backend: AcceleratorBackend,
    loaded_model_id: Arc<RwLock<Option<String>>>,
    target_resolution: usize,
}

impl VisionAdapter {
    /// Creates a new `VisionAdapter` with CPU execution baseline.
    pub fn new(adapter_id: impl Into<String>) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            engine: Arc::new(RwLock::new(Some(VisionEngine::default()))),
            active_backend: AcceleratorBackend::Cpu,
            loaded_model_id: Arc::new(RwLock::new(Some("builtin_vision_cpu_v1".to_string()))),
            target_resolution: DEFAULT_VISION_RESOLUTION,
        }
    }

    /// Sets the target resolution for tensor resizing.
    pub fn with_resolution(mut self, resolution: usize) -> Self {
        self.target_resolution = resolution;
        self.engine = Arc::new(RwLock::new(Some(VisionEngine::new(resolution))));
        self
    }

    /// Verifies SHA-256 and loads external vision model weights into the engine.
    pub fn load_model_weights(
        &self,
        weights_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<(), VisionError> {
        if !weights_path.is_file() {
            return Err(VisionError::IoError(format!(
                "Vision weights file not found: {}",
                weights_path.display()
            )));
        }

        let bytes = std::fs::read(weights_path)
            .map_err(|e| VisionError::IoError(format!("Failed to read weights: {e}")))?;

        if let Some(expected) = expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected {
                return Err(VisionError::CorruptPayload {
                    details: format!(
                        "Vision model SHA-256 mismatch: expected {}, got {}",
                        expected, actual
                    ),
                });
            }
        }

        let model_id = weights_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "custom_vision_model".to_string());

        let mut engine_guard = self
            .engine
            .write()
            .map_err(|_| VisionError::InferenceFailed {
                details: "Lock poisoned on vision engine".to_string(),
            })?;
        *engine_guard = Some(VisionEngine::new(self.target_resolution));

        let mut id_guard =
            self.loaded_model_id
                .write()
                .map_err(|_| VisionError::InferenceFailed {
                    details: "Lock poisoned on model id".to_string(),
                })?;
        *id_guard = Some(model_id);

        Ok(())
    }

    /// Unloads vision engine and releases memory.
    pub fn unload_model(&self) {
        if let Ok(mut guard) = self.engine.write() {
            *guard = None;
        }
        if let Ok(mut id_guard) = self.loaded_model_id.write() {
            *id_guard = None;
        }
    }

    /// Checks if a vision model is currently loaded.
    pub fn is_loaded(&self) -> bool {
        self.engine.read().map(|g| g.is_some()).unwrap_or(false)
    }

    /// Sanitizes image headers and returns metadata.
    pub fn sanitize_image(&self, bytes: &[u8]) -> Result<ImageMetadata, VisionError> {
        ImageSanitizer::sanitize(bytes)
    }

    /// Generates high-level visual description for an image.
    pub fn describe_image(
        &self,
        image_bytes: &[u8],
        raw_rgb: &[u8],
    ) -> Result<VisualDescription, VisionError> {
        let meta = self.sanitize_image(image_bytes)?;
        let tensor = ImageTensor::from_rgb_bytes(
            raw_rgb,
            meta.dimensions.width as usize,
            meta.dimensions.height as usize,
            self.target_resolution,
            self.target_resolution,
        )?;

        let engine_guard = self
            .engine
            .read()
            .map_err(|_| VisionError::InferenceFailed {
                details: "Lock poisoned on vision engine".to_string(),
            })?;
        let engine = engine_guard.as_ref().ok_or(VisionError::ModelNotLoaded)?;
        Ok(engine.describe_scene(&meta, &tensor))
    }

    /// Performs OCR text extraction with bounding boxes.
    pub fn extract_text_ocr(
        &self,
        image_bytes: &[u8],
        raw_rgb: &[u8],
    ) -> Result<OcrResult, VisionError> {
        let meta = self.sanitize_image(image_bytes)?;
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| VisionError::InferenceFailed {
                details: "Lock poisoned on vision engine".to_string(),
            })?;
        let engine = engine_guard.as_ref().ok_or(VisionError::ModelNotLoaded)?;
        engine.perform_ocr(raw_rgb, meta.dimensions)
    }

    /// Performs Visual Question Answering on an image.
    pub fn answer_vqa(
        &self,
        image_bytes: &[u8],
        raw_rgb: &[u8],
        query: &VisualQaQuery,
    ) -> Result<VisualQaAnswer, VisionError> {
        let meta = self.sanitize_image(image_bytes)?;
        let tensor = ImageTensor::from_rgb_bytes(
            raw_rgb,
            meta.dimensions.width as usize,
            meta.dimensions.height as usize,
            self.target_resolution,
            self.target_resolution,
        )?;

        let engine_guard = self
            .engine
            .read()
            .map_err(|_| VisionError::InferenceFailed {
                details: "Lock poisoned on vision engine".to_string(),
            })?;
        let engine = engine_guard.as_ref().ok_or(VisionError::ModelNotLoaded)?;
        let ocr = engine.perform_ocr(raw_rgb, meta.dimensions).ok();
        Ok(engine.answer_visual_question(query, &meta, &tensor, ocr.as_ref()))
    }
}

impl ModalityAdapter for VisionAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn modality(&self) -> Modality {
        Modality::Vision
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![AcceleratorBackend::Cpu]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::Vision,
            backends: vec![AcceleratorBackend::Cpu],
            supports_streaming: false,
            supports_offline: true,
            isolated_process: true,
        }
    }

    fn health_check(&self) -> Result<AdapterHealth, MiniAiError> {
        let loaded = self.is_loaded();
        let model_name = self
            .loaded_model_id
            .read()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_else(|| "none".to_string());

        Ok(AdapterHealth {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::Vision,
            ready: loaded,
            active_backend: self.active_backend,
            status_message: if loaded {
                format!("Vision adapter active on CPU (model: {model_name})")
            } else {
                "Vision adapter initialized but no model loaded".to_string()
            },
        })
    }
}
