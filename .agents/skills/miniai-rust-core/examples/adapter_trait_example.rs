// Reference example: Implementing a CPU-first ModalityAdapter in MiniAI Rust Core
use miniai_core::{
    AcceleratorBackend, AdapterHealth, MiniAiError, Modality, ModalityAdapter, RuntimeCapability,
};

pub struct CpuReferenceLlmAdapter {
    id: String,
    loaded: bool,
}

impl CpuReferenceLlmAdapter {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            loaded: false,
        }
    }
}

impl ModalityAdapter for CpuReferenceLlmAdapter {
    fn adapter_id(&self) -> &str {
        &self.id
    }

    fn modality(&self) -> Modality {
        Modality::Llm
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![AcceleratorBackend::Cpu]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.id.clone(),
            modality: Modality::Llm,
            backends: vec![AcceleratorBackend::Cpu],
            supports_streaming: true,
            supports_offline: true,
            isolated_process: true,
        }
    }

    fn health_check(&self) -> Result<AdapterHealth, MiniAiError> {
        Ok(AdapterHealth {
            adapter_id: self.id.clone(),
            ready: true,
            active_backend: AcceleratorBackend::Cpu,
            status_message: if self.loaded {
                "Model loaded on CPU".to_string()
            } else {
                "Adapter initialized (idle on CPU)".to_string()
            },
        })
    }
}

