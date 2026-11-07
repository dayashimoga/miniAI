use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use miniai_core::{
    AcceleratorBackend, AdapterHealth, MiniAiError, Modality, ModalityAdapter, RuntimeCapability,
};
use sha2::{Digest, Sha256};

use crate::engine::MusicGenEngine;
use crate::error::MusicError;
use crate::types::{MusicGenConfig, MusicProgress, MusicTrackOutput};

/// Pluggable offline polyphonic music synthesis adapter implementing `ModalityAdapter`.
pub struct MusicAdapter {
    adapter_id: String,
    engine: Arc<RwLock<Option<MusicGenEngine>>>,
    active_backend: AcceleratorBackend,
    loaded_model_id: Arc<RwLock<Option<String>>>,
}

impl MusicAdapter {
    /// Creates a new `MusicAdapter` with default built-in CPU model.
    pub fn new(adapter_id: impl Into<String>) -> Self {
        let mut engine = MusicGenEngine::new("builtin_music_polyphonic_cpu_v1");
        engine.set_weights_loaded(true);

        Self {
            adapter_id: adapter_id.into(),
            engine: Arc::new(RwLock::new(Some(engine))),
            active_backend: AcceleratorBackend::Cpu,
            loaded_model_id: Arc::new(RwLock::new(Some(
                "builtin_music_polyphonic_cpu_v1".to_string(),
            ))),
        }
    }

    /// Loads music synthesis weights from disk with SHA-256 validation.
    pub fn load_model_weights(
        &self,
        weights_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<(), MusicError> {
        if !weights_path.is_file() {
            return Err(MusicError::Io(format!(
                "Music weights file not found: {}",
                weights_path.display()
            )));
        }

        let bytes = std::fs::read(weights_path)
            .map_err(|e| MusicError::Io(format!("Failed to read music weights: {e}")))?;

        if let Some(expected) = expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected {
                return Err(MusicError::CorruptPayload(format!(
                    "Music model SHA-256 mismatch: expected {}, got {}",
                    expected, actual
                )));
            }
        }

        let model_id = weights_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "custom_music_model".to_string());

        let mut engine = MusicGenEngine::new(&model_id);
        engine.set_weights_loaded(true);

        let mut engine_guard = self
            .engine
            .write()
            .map_err(|_| MusicError::Processing("Lock poisoned on music engine".to_string()))?;
        *engine_guard = Some(engine);

        let mut id_guard = self
            .loaded_model_id
            .write()
            .map_err(|_| MusicError::Processing("Lock poisoned on model id".to_string()))?;
        *id_guard = Some(model_id);

        Ok(())
    }

    /// Unloads the music engine from memory.
    pub fn unload_model(&self) {
        if let Ok(mut guard) = self.engine.write() {
            *guard = None;
        }
        if let Ok(mut id_guard) = self.loaded_model_id.write() {
            *id_guard = None;
        }
    }

    /// Checks if a music synthesis model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.engine
            .read()
            .map(|g| g.as_ref().map(|e| e.is_weights_loaded()).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Synthesizes polyphonic music with the specified configuration.
    pub fn synthesize(&self, config: &MusicGenConfig) -> Result<MusicTrackOutput, MusicError> {
        self.synthesize_with_cancellation(config, None, None)
    }

    /// Synthesizes polyphonic music with optional cancellation and progress telemetry.
    pub fn synthesize_with_cancellation(
        &self,
        config: &MusicGenConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
        progress_cb: Option<&dyn Fn(MusicProgress)>,
    ) -> Result<MusicTrackOutput, MusicError> {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| MusicError::Processing("Lock poisoned on music engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(MusicError::UnloadedModel)?;

        engine.synthesize(config, cancel_token, progress_cb)
    }
}

impl ModalityAdapter for MusicAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn modality(&self) -> Modality {
        Modality::Music
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![AcceleratorBackend::Cpu]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::Music,
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
            modality: Modality::Music,
            ready: loaded,
            active_backend: self.active_backend,
            status_message: if loaded {
                format!("Music adapter active on CPU (model: {model_name})")
            } else {
                "Music adapter initialized but no model loaded".to_string()
            },
        })
    }
}
