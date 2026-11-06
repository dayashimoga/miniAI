use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;

use miniai_core::{AcceleratorBackend, AdapterHealth, Modality};
use miniai_server::{EventBus, SystemEvent};

use crate::config::{GenerationConfig, GenerationOutput, StreamToken};
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

    pub fn unload_model(&mut self) {
        self.engine.unload_model();
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
