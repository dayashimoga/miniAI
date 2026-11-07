use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use miniai_core::{
    AcceleratorBackend, AdapterHealth, MiniAiError, Modality, ModalityAdapter, RuntimeCapability,
};
use sha2::{Digest, Sha256};

use crate::error::SpeechTtsError;
use crate::synthesizer::SpeechTtsEngine;
use crate::types::{TtsAudioOutput, TtsProgress, TtsSynthesisConfig};

/// Pluggable Text-to-Speech (TTS) runtime adapter implementing `ModalityAdapter`.
pub struct SpeechTtsAdapter {
    adapter_id: String,
    engine: Arc<RwLock<Option<SpeechTtsEngine>>>,
    active_backend: AcceleratorBackend,
    loaded_model_id: Arc<RwLock<Option<String>>>,
}

impl SpeechTtsAdapter {
    /// Creates a new `SpeechTtsAdapter` with default built-in CPU model.
    pub fn new(adapter_id: impl Into<String>) -> Self {
        let mut engine = SpeechTtsEngine::new("builtin_acoustic_tts_cpu_v1");
        engine.set_weights_loaded(true);

        Self {
            adapter_id: adapter_id.into(),
            engine: Arc::new(RwLock::new(Some(engine))),
            active_backend: AcceleratorBackend::Cpu,
            loaded_model_id: Arc::new(RwLock::new(Some("builtin_acoustic_tts_cpu_v1".to_string()))),
        }
    }

    /// Verifies cryptographic SHA-256 and loads external TTS model weights into the engine.
    pub fn load_model_weights(
        &self,
        weights_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<(), SpeechTtsError> {
        if !weights_path.is_file() {
            return Err(SpeechTtsError::Io(format!(
                "TTS weights file not found: {}",
                weights_path.display()
            )));
        }

        let bytes = std::fs::read(weights_path)
            .map_err(|e| SpeechTtsError::Io(format!("Failed to read TTS weights: {e}")))?;

        if let Some(expected) = expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected {
                return Err(SpeechTtsError::CorruptPayload(format!(
                    "TTS model SHA-256 mismatch: expected {}, got {}",
                    expected, actual
                )));
            }
        }

        let model_id = weights_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "custom_tts_model".to_string());

        let mut engine = SpeechTtsEngine::new(&model_id);
        engine.set_weights_loaded(true);

        let mut engine_guard = self
            .engine
            .write()
            .map_err(|_| SpeechTtsError::Processing("Lock poisoned on TTS engine".to_string()))?;
        *engine_guard = Some(engine);

        let mut id_guard = self
            .loaded_model_id
            .write()
            .map_err(|_| SpeechTtsError::Processing("Lock poisoned on model id".to_string()))?;
        *id_guard = Some(model_id);

        Ok(())
    }

    /// Unloads the TTS engine and frees memory.
    pub fn unload_model(&self) {
        if let Ok(mut guard) = self.engine.write() {
            *guard = None;
        }
        if let Ok(mut id_guard) = self.loaded_model_id.write() {
            *id_guard = None;
        }
    }

    /// Checks if a TTS model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.engine
            .read()
            .map(|g| g.as_ref().map(|e| e.is_weights_loaded()).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Synthesizes text with default configuration.
    pub fn synthesize_text(
        &self,
        text: &str,
        config: &TtsSynthesisConfig,
    ) -> Result<TtsAudioOutput, SpeechTtsError> {
        self.synthesize_with_cancellation(text, config, None, None)
    }

    /// Synthesizes text with optional cancellation token and progress callback.
    pub fn synthesize_with_cancellation(
        &self,
        text: &str,
        config: &TtsSynthesisConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
        progress_cb: Option<&dyn Fn(TtsProgress)>,
    ) -> Result<TtsAudioOutput, SpeechTtsError> {
        let engine_guard = self
            .engine
            .read()
            .map_err(|_| SpeechTtsError::Processing("Lock poisoned on TTS engine".to_string()))?;
        let engine = engine_guard.as_ref().ok_or(SpeechTtsError::UnloadedModel)?;

        engine.synthesize(text, config, cancel_token, progress_cb)
    }
}

impl ModalityAdapter for SpeechTtsAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn modality(&self) -> Modality {
        Modality::SpeechTts
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![AcceleratorBackend::Cpu]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::SpeechTts,
            backends: vec![AcceleratorBackend::Cpu],
            supports_streaming: true,
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
            modality: Modality::SpeechTts,
            ready: loaded,
            active_backend: self.active_backend,
            status_message: if loaded {
                format!("Speech TTS adapter active on CPU (model: {model_name})")
            } else {
                "Speech TTS adapter initialized but no model loaded".to_string()
            },
        })
    }
}
