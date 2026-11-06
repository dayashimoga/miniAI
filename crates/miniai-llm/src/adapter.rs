use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;

use miniai_core::{AcceleratorBackend, AdapterHealth, AdmissionGuard, Modality};
use miniai_server::{EventBus, SystemEvent};

use crate::config::{GenerationConfig, GenerationOutput, StreamToken, SwitchReport};
use crate::engine::LlmEngine;
use crate::error::LlmError;

/// Adapter integrating the LLM inference engine with the MiniAI runtime and event bus.
#[derive(Debug, Clone)]
pub struct LlmAdapter {
    adapter_id: String,
    engine: LlmEngine,
}

impl Default for LlmAdapter {
    fn default() -> Self {
        Self::new("miniai-llm-default")
    }
}

impl LlmAdapter {
    pub fn new(adapter_id: impl Into<String>) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            engine: LlmEngine::new(),
        }
    }

    pub fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    pub fn modality(&self) -> Modality {
        Modality::Llm
    }

    pub fn is_loaded(&self) -> bool {
        self.engine.is_loaded()
    }

    pub fn active_backend(&self) -> AcceleratorBackend {
        self.engine.active_backend()
    }

    pub fn load_model(
        &mut self,
        model_id: &str,
        path: &Path,
        backend: AcceleratorBackend,
    ) -> Result<(), LlmError> {
        self.engine.load_model(model_id, path, backend)
    }

    pub fn unload_model(&mut self) -> u64 {
        self.engine.unload_model()
    }

    pub fn loaded_model_id(&self) -> Option<&str> {
        self.engine.loaded_model_id()
    }

    pub fn memory_footprint_bytes(&self) -> u64 {
        self.engine.memory_footprint_bytes()
    }

    /// Hot-switches the model with verified memory release.
    pub fn switch_model(
        &mut self,
        new_model_id: &str,
        new_model_path: &Path,
        backend: AcceleratorBackend,
    ) -> Result<SwitchReport, LlmError> {
        self.engine
            .switch_model(new_model_id, new_model_path, backend)
    }

    /// Hot-switches the model with verified memory release, admission guard evaluation,
    /// automatic CPU fallback, and event bus telemetry broadcast.
    pub fn switch_model_with_guard(
        &mut self,
        new_model_id: &str,
        new_model_path: &Path,
        preferred_backend: Option<AcceleratorBackend>,
        admission_guard: &AdmissionGuard,
        event_bus: Option<&EventBus>,
    ) -> Result<SwitchReport, LlmError> {
        let file_meta =
            std::fs::metadata(new_model_path).map_err(|e| LlmError::ModelLoadError {
                reason: format!("Failed to read replacement model file: {}", e),
            })?;
        let required_bytes = file_meta.len();

        let decision = admission_guard.evaluate_admission(
            required_bytes,
            required_bytes,
            preferred_backend,
        )?;

        let selected_backend = decision.selected_backend;

        if let Some(bus) = event_bus {
            let _ = bus.publish_now(SystemEvent::AdapterHealthChanged {
                health: AdapterHealth {
                    adapter_id: self.adapter_id.clone(),
                    modality: Modality::Llm,
                    ready: false,
                    active_backend: selected_backend,
                    status_message: format!("Hot-switching to model '{}'", new_model_id),
                },
            });
        }

        let report = self
            .engine
            .switch_model(new_model_id, new_model_path, selected_backend)?;

        if let Some(bus) = event_bus {
            let _ = bus.publish_now(SystemEvent::AdapterHealthChanged {
                health: self.health_check(),
            });
        }

        Ok(report)
    }

    pub fn health_check(&self) -> AdapterHealth {
        let loaded = self.engine.is_loaded();
        AdapterHealth {
            adapter_id: self.adapter_id.clone(),
            modality: Modality::Llm,
            ready: loaded,
            active_backend: self.engine.active_backend(),
            status_message: if loaded {
                format!(
                    "LLM adapter active on {:?} (footprint: {} bytes)",
                    self.engine.active_backend(),
                    self.engine.memory_footprint_bytes()
                )
            } else {
                "LLM adapter ready (unloaded)".to_string()
            },
        }
    }

    /// Generates text completion synchronously.
    pub fn generate(
        &self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<GenerationOutput, LlmError> {
        self.engine.generate(prompt, config)
    }

    /// Generates text completion while streaming tokens through a Tokio MPSC channel.
    pub fn generate_stream_channel(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        cancel_token: Option<Arc<AtomicBool>>,
    ) -> Result<(GenerationOutput, mpsc::Receiver<StreamToken>), LlmError> {
        let (tx, rx) = mpsc::channel(64);
        let output = self
            .engine
            .generate_stream(prompt, config, cancel_token, |token| {
                tx.blocking_send(token).is_ok()
            })?;
        Ok((output, rx))
    }

    /// Generates text completion while dispatching `SystemEvent::InferenceTokenStream` events
    /// directly to the `EventBus` for live multi-subscriber SSE delivery.
    pub fn generate_stream_to_event_bus(
        &self,
        session_id: &str,
        prompt: &str,
        config: &GenerationConfig,
        cancel_token: Option<Arc<AtomicBool>>,
        event_bus: &EventBus,
    ) -> Result<GenerationOutput, LlmError> {
        let sid = session_id.to_string();
        self.engine
            .generate_stream(prompt, config, cancel_token, |token| {
                let event = SystemEvent::InferenceTokenStream {
                    session_id: sid.clone(),
                    token: token.text,
                    is_final: token.is_final,
                };
                let _ = event_bus.publish_now(event);
                true
            })
    }
}
