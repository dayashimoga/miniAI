use serde::{Deserialize, Serialize};

use crate::error::LlmError;

/// Terminal reason for token generation completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural end of sequence token encountered (e.g. `</s>`, `<|endoftext|>`).
    EndOfSequence,
    /// One of the configured stop sequences was matched.
    StopSequence,
    /// Maximum new tokens reached.
    Length,
    /// User cancelled the stream or request timed out.
    Cancelled,
}

/// Generation hyperparameters and sampling controls for LLM inference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Maximum number of tokens to generate.
    pub max_new_tokens: usize,

    /// Maximum context window size (prompt tokens + completion tokens).
    pub context_window_size: usize,

    /// Sampling temperature (0.0 = greedy argmax, >0.0 = stochastic scaling).
    pub temperature: f32,

    /// Top-p (nucleus) cumulative probability threshold in (0.0, 1.0].
    pub top_p: f32,

    /// Repetition penalty factor applied to previously emitted tokens (1.0 = none).
    pub repetition_penalty: f32,

    /// Optional deterministic PRNG seed for exact reproducibility.
    pub seed: Option<u64>,

    /// String sequences that trigger generation termination upon matching.
    pub stop_sequences: Vec<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 256,
            context_window_size: 2048,
            temperature: 0.7,
            top_p: 0.9,
            repetition_penalty: 1.1,
            seed: None,
            stop_sequences: vec!["\n\nUser:".to_string(), "</s>".to_string()],
        }
    }
}

impl GenerationConfig {
    /// Validates all generation parameters against safe domain ranges.
    pub fn validate(&self) -> Result<(), LlmError> {
        if self.max_new_tokens == 0 {
            return Err(LlmError::InvalidConfig {
                reason: "max_new_tokens must be greater than 0".to_string(),
            });
        }
        if self.context_window_size < 16 {
            return Err(LlmError::InvalidConfig {
                reason: "context_window_size must be at least 16 tokens".to_string(),
            });
        }
        if self.temperature < 0.0 || self.temperature > 5.0 {
            return Err(LlmError::InvalidConfig {
                reason: format!("temperature {} out of range [0.0, 5.0]", self.temperature),
            });
        }
        if self.top_p <= 0.0 || self.top_p > 1.0 {
            return Err(LlmError::InvalidConfig {
                reason: format!("top_p {} out of range (0.0, 1.0]", self.top_p),
            });
        }
        if self.repetition_penalty < 0.5 || self.repetition_penalty > 3.0 {
            return Err(LlmError::InvalidConfig {
                reason: format!(
                    "repetition_penalty {} out of range [0.5, 3.0]",
                    self.repetition_penalty
                ),
            });
        }
        Ok(())
    }
}

/// Telemetry and text output from a completed generation run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationOutput {
    /// Final generated text.
    pub text: String,

    /// List of generated token IDs.
    pub tokens: Vec<u32>,

    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,

    /// Number of tokens generated in the completion.
    pub completion_tokens: usize,

    /// Time to first token in milliseconds.
    pub ttft_ms: f64,

    /// Total generation wall-clock time in milliseconds.
    pub total_time_ms: f64,

    /// Generation throughput in tokens per second.
    pub tokens_per_second: f64,

    /// Reason generation stopped.
    pub finish_reason: FinishReason,
}

/// Token payload emitted during real-time streaming.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamToken {
    /// Raw token ID.
    pub token_id: u32,

    /// Decoded token text string.
    pub text: String,

    /// Monotonic token index within the current completion stream.
    pub index: usize,

    /// Whether this is the final token of the generation.
    pub is_final: bool,

    /// Terminal reason if `is_final` is true.
    pub finish_reason: Option<FinishReason>,
}
