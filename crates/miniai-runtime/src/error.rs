use miniai_core::{AcceleratorBackend, MiniAiError, Modality};
use thiserror::Error;

/// Errors produced by the modality runtime supervisor and adapter subsystem.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Adapter error for modality {modality:?}: {message}")]
    AdapterError { modality: Modality, message: String },

    #[error("Model not loaded for modality {modality:?}")]
    ModelNotLoaded { modality: Modality },

    #[error("Model already loaded for modality {modality:?}: active model '{model_id}'")]
    ModelAlreadyLoaded {
        modality: Modality,
        model_id: String,
    },

    #[error("Core error: {0}")]
    CoreError(#[from] MiniAiError),

    #[error("Storage error: {0}")]
    StorageError(#[from] miniai_storage::error::StorageError),

    #[error("Registry error: {0}")]
    RegistryError(#[from] miniai_registry::RegistryError),

    #[error(
        "Accelerator '{backend:?}' failed for modality {modality:?}: {reason}; CPU fallback executed"
    )]
    AcceleratorFailedWithCpuFallback {
        modality: Modality,
        backend: AcceleratorBackend,
        reason: String,
    },
}
