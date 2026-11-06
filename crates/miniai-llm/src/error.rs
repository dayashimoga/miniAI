use miniai_core::MiniAiError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors occurring within the MiniAI LLM inference engine.
#[derive(Debug, Error, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "error_type", content = "details", rename_all = "snake_case")]
pub enum LlmError {
    #[error("No LLM model is currently loaded in memory")]
    ModelNotLoaded,

    #[error("Context window exceeded: prompt has {tokens} tokens, max context is {max_context}")]
    ContextWindowExceeded { tokens: usize, max_context: usize },

    #[error("Tokenization failure: {reason}")]
    TokenizationError { reason: String },

    #[error("Inference generation was cancelled by user or timeout")]
    InferenceCancelled,

    #[error("Failed to load model weights: {reason}")]
    ModelLoadError { reason: String },

    #[error("Invalid generation parameter: {reason}")]
    InvalidConfig { reason: String },

    #[error("MiniAI core error: {0}")]
    Core(#[from] MiniAiError),
}
