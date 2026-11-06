use std::path::{Path, PathBuf};
use std::time::SystemTime;

use miniai_core::{AcceleratorBackend, AdapterHealth, Modality, RuntimeCapability};
use serde::{Deserialize, Serialize};

use crate::error::RuntimeError;

/// Lifecycle state of an individual modality adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum AdapterState {
    Unloaded,
    Loading {
        model_id: String,
    },
    Loaded {
        model_id: String,
        model_path: PathBuf,
        active_backend: AcceleratorBackend,
        loaded_at_epoch_secs: u64,
        memory_footprint_bytes: u64,
    },
    Degraded {
        model_id: String,
        active_backend: AcceleratorBackend,
        reason: String,
    },
    Failed {
        error: String,
    },
}

/// An independently managed, process-isolated modality adapter instance.
#[derive(Debug, Clone)]
pub struct ManagedAdapter {
    adapter_id: String,
    modality: Modality,
    state: AdapterState,
    supported_backends: Vec<AcceleratorBackend>,
    supports_streaming: bool,
}

impl ManagedAdapter {
    /// Creates a new managed adapter for a given modality with CPU-first baseline support.
    pub fn new(
        adapter_id: impl Into<String>,
        modality: Modality,
        supports_streaming: bool,
    ) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            modality,
            state: AdapterState::Unloaded,
            supported_backends: vec![
                AcceleratorBackend::Cpu,
                AcceleratorBackend::Cuda,
                AcceleratorBackend::Vulkan,
                AcceleratorBackend::Metal,
            ],
            supports_streaming,
        }
    }

    pub fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    pub fn modality(&self) -> Modality {
        self.modality
    }

    pub fn state(&self) -> &AdapterState {
        &self.state
    }

    pub fn supported_backends(&self) -> &[AcceleratorBackend] {
        &self.supported_backends
    }

    pub fn supports_streaming(&self) -> bool {
        self.supports_streaming
    }

    /// Loads a verified model artifact into this adapter.
    pub fn load_model(
        &mut self,
        model_id: &str,
        model_path: &Path,
        backend: AcceleratorBackend,
        memory_bytes: u64,
    ) -> Result<(), RuntimeError> {
        if let AdapterState::Loaded {
            model_id: ref current,
            ..
        } = self.state
        {
            return Err(RuntimeError::ModelAlreadyLoaded {
                modality: self.modality,
                model_id: current.clone(),
            });
        }

        let now_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.state = AdapterState::Loaded {
            model_id: model_id.to_string(),
            model_path: model_path.to_path_buf(),
            active_backend: backend,
            loaded_at_epoch_secs: now_epoch,
            memory_footprint_bytes: memory_bytes,
        };

        Ok(())
    }

    /// Unloads the active model from memory.
    pub fn unload_model(&mut self) -> Result<(), RuntimeError> {
        self.state = AdapterState::Unloaded;
        Ok(())
    }

    /// Demotes active execution backend to `AcceleratorBackend::Cpu` due to failure or request.
    pub fn fallback_to_cpu(&mut self, reason: &str) -> Result<AdapterHealth, RuntimeError> {
        match &self.state {
            AdapterState::Loaded {
                model_id,
                model_path,
                loaded_at_epoch_secs,
                memory_footprint_bytes,
                ..
            } => {
                let current_id = model_id.clone();
                let current_path = model_path.clone();
                let epoch = *loaded_at_epoch_secs;
                let mem = *memory_footprint_bytes;

                self.state = AdapterState::Degraded {
                    model_id: current_id.clone(),
                    active_backend: AcceleratorBackend::Cpu,
                    reason: reason.to_string(),
                };

                // Keep model loaded on CPU baseline
                self.state = AdapterState::Loaded {
                    model_id: current_id,
                    model_path: current_path,
                    active_backend: AcceleratorBackend::Cpu,
                    loaded_at_epoch_secs: epoch,
                    memory_footprint_bytes: mem,
                };

                Ok(self.health_check())
            }
            AdapterState::Degraded { .. } | AdapterState::Loading { .. } => {
                self.state = AdapterState::Degraded {
                    model_id: "unknown".to_string(),
                    active_backend: AcceleratorBackend::Cpu,
                    reason: reason.to_string(),
                };
                Ok(self.health_check())
            }
            AdapterState::Unloaded => {
                // If unloaded, fallback just ensures CPU is ready
                Ok(self.health_check())
            }
            AdapterState::Failed { .. } => {
                // Recover to unloaded CPU
                self.state = AdapterState::Unloaded;
                Ok(self.health_check())
            }
        }
    }

    /// Computes and returns the health status of this adapter.
    pub fn health_check(&self) -> AdapterHealth {
        match &self.state {
            AdapterState::Unloaded => AdapterHealth {
                adapter_id: self.adapter_id.clone(),
                modality: self.modality,
                ready: true,
                active_backend: AcceleratorBackend::Cpu,
                status_message: format!(
                    "{} adapter ready (no model loaded)",
                    self.modality.display_name()
                ),
            },
            AdapterState::Loading { model_id } => AdapterHealth {
                adapter_id: self.adapter_id.clone(),
                modality: self.modality,
                ready: false,
                active_backend: AcceleratorBackend::Cpu,
                status_message: format!(
                    "Loading model '{}' for {}",
                    model_id,
                    self.modality.display_name()
                ),
            },
            AdapterState::Loaded {
                model_id,
                active_backend,
                memory_footprint_bytes,
                ..
            } => AdapterHealth {
                adapter_id: self.adapter_id.clone(),
                modality: self.modality,
                ready: true,
                active_backend: *active_backend,
                status_message: format!(
                    "Model '{}' active on {:?} ({:.1} MB footprint)",
                    model_id,
                    active_backend,
                    *memory_footprint_bytes as f64 / (1024.0 * 1024.0)
                ),
            },
            AdapterState::Degraded {
                model_id,
                active_backend,
                reason,
            } => AdapterHealth {
                adapter_id: self.adapter_id.clone(),
                modality: self.modality,
                ready: true,
                active_backend: *active_backend,
                status_message: format!(
                    "Model '{}' degraded on {:?}: {}",
                    model_id, active_backend, reason
                ),
            },
            AdapterState::Failed { error } => AdapterHealth {
                adapter_id: self.adapter_id.clone(),
                modality: self.modality,
                ready: false,
                active_backend: AcceleratorBackend::Cpu,
                status_message: format!("Adapter failure: {}", error),
            },
        }
    }

    /// Returns the static and dynamic capability profile for this adapter.
    pub fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: self.modality,
            backends: self.supported_backends.clone(),
            supports_streaming: self.supports_streaming,
            supports_offline: true,
            isolated_process: true,
        }
    }
}
