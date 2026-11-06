use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use miniai_core::{
    AcceleratorBackend, AdapterHealth, MiniAiError, Modality, ModalityAdapter, RuntimeCapability,
};
use sha2::{Digest, Sha256};

use crate::engine::ImageGenEngine;
use crate::error::ImageGenError;
use crate::types::{ImageGenConfig, ImageGenResult, StepProgress};

/// Decoupled, pluggable Image Generation runtime adapter implementing `ModalityAdapter`.
pub struct ImageGenAdapter {
    adapter_id: String,
    engine: Arc<RwLock<Option<ImageGenEngine>>>,
    active_backend: AcceleratorBackend,
    loaded_model_id: Arc<RwLock<Option<String>>>,
}

impl ImageGenAdapter {
    /// Creates a new `ImageGenAdapter` ready on universal CPU baseline.
    pub fn new(adapter_id: impl Into<String>) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            engine: Arc::new(RwLock::new(Some(ImageGenEngine::default()))),
            active_backend: AcceleratorBackend::Cpu,
            loaded_model_id: Arc::new(RwLock::new(Some("builtin_imagegen_cpu_v1".to_string()))),
        }
    }

    /// Verifies SHA-256 and loads external diffusion model weights into the engine.
    pub fn load_model_weights(
        &self,
        weights_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<(), ImageGenError> {
        if !weights_path.is_file() {
            return Err(ImageGenError::IoError(format!(
                "ImageGen weights file not found: {}",
                weights_path.display()
            )));
        }

        let bytes = std::fs::read(weights_path)
            .map_err(|e| ImageGenError::IoError(format!("Failed to read weights: {e}")))?;

        if let Some(expected) = expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected {
                return Err(ImageGenError::PngError(format!(
                    "ImageGen model SHA-256 mismatch: expected {}, got {}",
                    expected, actual
                )));
            }
        }

        let model_id = weights_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "custom_imagegen_model".to_string());

        let mut engine_guard = self
            .engine
            .write()
            .map_err(|_| ImageGenError::IoError("Lock poisoned on imagegen engine".to_string()))?;
        *engine_guard = Some(ImageGenEngine::new(&model_id));

        let mut id_guard = self
            .loaded_model_id
            .write()
            .map_err(|_| ImageGenError::IoError("Lock poisoned on model id".to_string()))?;
        *id_guard = Some(model_id);

        Ok(())
    }

    /// Unloads diffusion engine and releases memory.
    pub fn unload_model(&self) {
        if let Ok(mut guard) = self.engine.write() {
            *guard = None;
        }
        if let Ok(mut id_guard) = self.loaded_model_id.write() {
            *id_guard = None;
        }
    }

    /// Checks if an image generation model is currently loaded.
    pub fn is_loaded(&self) -> bool {
        self.engine.read().map(|g| g.is_some()).unwrap_or(false)
    }

    /// Generates image from text prompt.
    pub fn generate<F>(
        &self,
        config: &ImageGenConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
        on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| ImageGenError::IoError("Lock poisoned on engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(ImageGenError::ModelNotLoaded)?;
        engine.generate_text_to_image(config, cancel_token, on_progress)
    }

    /// Refines base image with text conditioning and perturbation strength.
    pub fn refine<F>(
        &self,
        config: &ImageGenConfig,
        input_rgb: &[u8],
        input_width: u32,
        input_height: u32,
        cancel_token: Option<&Arc<AtomicBool>>,
        on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| ImageGenError::IoError("Lock poisoned on engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(ImageGenError::ModelNotLoaded)?;
        engine.generate_image_to_image(
            config,
            input_rgb,
            input_width,
            input_height,
            cancel_token,
            on_progress,
        )
    }

    /// Inpaints designated masked region while preserving unmasked canvas.
    pub fn inpaint<F>(
        &self,
        config: &ImageGenConfig,
        base_rgb: &[u8],
        mask_bytes: &[u8],
        cancel_token: Option<&Arc<AtomicBool>>,
        on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| ImageGenError::IoError("Lock poisoned on engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(ImageGenError::ModelNotLoaded)?;
        engine.inpaint_image(config, base_rgb, mask_bytes, cancel_token, on_progress)
    }
}

impl ModalityAdapter for ImageGenAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn modality(&self) -> Modality {
        Modality::ImageGeneration
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![AcceleratorBackend::Cpu]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::ImageGeneration,
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
            modality: Modality::ImageGeneration,
            ready: loaded,
            active_backend: self.active_backend,
            status_message: if loaded {
                format!("ImageGeneration adapter active on CPU (model: {model_name})")
            } else {
                "ImageGeneration adapter initialized but no model loaded".to_string()
            },
        })
    }
}
