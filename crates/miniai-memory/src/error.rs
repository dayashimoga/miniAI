use thiserror::Error;

/// Errors produced during memory lifecycle, retrieval, and eviction.
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Storage error: {0}")]
    Storage(#[from] miniai_storage::StorageError),

    #[error("Working memory budget exceeded: required {required} tokens, budget is {budget}")]
    WorkingMemoryBudgetExceeded { required: usize, budget: usize },

    #[error("Memory entry not found: {entity} with id '{id}'")]
    NotFound { entity: &'static str, id: String },

    #[error("Serialization error: {0}")]
    Serialization(String),
}
