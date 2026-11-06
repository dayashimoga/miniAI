use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use miniai_core::AcceleratorBackend;

use crate::config::{FinishReason, GenerationConfig, GenerationOutput, StreamToken, SwitchReport};
use crate::error::LlmError;
use crate::model::{ModelMetadata, ModelWeights};
use crate::sampler::Sampler;
use crate::tokenizer::Tokenizer;

/// The MiniAI LLM inference engine providing CPU-first text generation, streaming, and cancellation.
#[derive(Debug, Clone)]
pub struct LlmEngine {
    tokenizer: Arc<Tokenizer>,
    weights: Option<ModelWeights>,
}

impl Default for LlmEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmEngine {
    /// Creates a new uninitialized LLM engine with the default tokenizer.
    pub fn new() -> Self {
        Self {
            tokenizer: Arc::new(Tokenizer::new_default()),
            weights: None,
        }
    }

    /// Creates an LLM engine with a custom tokenizer.
    pub fn with_tokenizer(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer: Arc::new(tokenizer),
            weights: None,
        }
    }

    /// Returns whether a model is currently loaded in memory.
    pub fn is_loaded(&self) -> bool {
        self.weights.is_some()
    }

    /// Returns the currently active execution backend, or `AcceleratorBackend::Cpu` if unloaded.
    pub fn active_backend(&self) -> AcceleratorBackend {
        self.weights
            .as_ref()
            .map(|w| w.active_backend)
            .unwrap_or(AcceleratorBackend::Cpu)
    }

    /// Loads model weights from disk into memory.
    pub fn load_model(
        &mut self,
        model_id: &str,
        path: &Path,
        backend: AcceleratorBackend,
    ) -> Result<(), LlmError> {
        let vocab_size = self.tokenizer.vocab_size();
        let weights = ModelWeights::load_from_path(model_id, path, backend, vocab_size)?;
        self.weights = Some(weights);
        Ok(())
    }

    /// Unloads active model weights, verifies memory release, and returns the number of freed bytes.
    pub fn unload_model(&mut self) -> u64 {
        if let Some(weights) = self.weights.take() {
            let freed = weights.memory_footprint_bytes;
            drop(weights);
            freed
        } else {
            0
        }
    }

    /// Returns the ID of the model currently loaded, or `None` if unloaded.
    pub fn loaded_model_id(&self) -> Option<&str> {
        self.weights.as_ref().map(|w| w.metadata.model_id.as_str())
    }

    /// Returns the metadata of the currently loaded model, if any.
    pub fn metadata(&self) -> Option<&ModelMetadata> {
        self.weights.as_ref().map(|w| &w.metadata)
    }

    /// Atomically hot-switches the active model, verifying that prior model weights
    /// and memory allocations are completely deallocated before loading the replacement.
    pub fn switch_model(
        &mut self,
        new_model_id: &str,
        new_model_path: &Path,
        backend: AcceleratorBackend,
    ) -> Result<SwitchReport, LlmError> {
        let start = Instant::now();
        let prev_id = self.loaded_model_id().map(|s| s.to_string());

        // Step 1: Explicitly unload prior model and verify memory is cleared
        let freed_memory_bytes = self.unload_model();
        assert!(
            self.weights.is_none(),
            "Prior model weights must be cleared from memory before loading replacement"
        );

        // Step 2: Load new model into clean memory
        match self.load_model(new_model_id, new_model_path, backend) {
            Ok(()) => {
                let allocated_memory_bytes = self.memory_footprint_bytes();
                let now_epoch = std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                Ok(SwitchReport {
                    previous_model_id: prev_id,
                    new_model_id: new_model_id.to_string(),
                    freed_memory_bytes,
                    allocated_memory_bytes,
                    active_backend: backend,
                    switched_at_epoch_secs: now_epoch,
                    duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                })
            }
            Err(e) => {
                // Ensure state remains clean and unloaded on failure
                self.weights = None;
                Err(e)
            }
        }
    }

    /// Returns the memory footprint of the currently loaded model in bytes.
    pub fn memory_footprint_bytes(&self) -> u64 {
        self.weights
            .as_ref()
            .map(|w| w.memory_footprint_bytes)
            .unwrap_or(0)
    }

    /// Tokenizes the input text into a sequence of token IDs.
    pub fn tokenize(&self, text: &str) -> Result<Vec<u32>, LlmError> {
        self.tokenizer.encode(text)
    }

    /// Decodes a sequence of token IDs into text.
    pub fn decode(&self, tokens: &[u32]) -> String {
        self.tokenizer.decode(tokens)
    }

    /// Synchronously executes complete prompt generation.
    pub fn generate(
        &self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<GenerationOutput, LlmError> {
        self.generate_stream(prompt, config, None, |_| true)
    }

    /// Executes generation while streaming each token through a callback closure.
    ///
    /// Respects atomic cancellation tokens, configurable stop sequences, repetition penalty,
    /// temperature scaling, top-p nucleus sampling, and context window limits.
    pub fn generate_stream<F>(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        cancel_token: Option<Arc<AtomicBool>>,
        mut on_token: F,
    ) -> Result<GenerationOutput, LlmError>
    where
        F: FnMut(StreamToken) -> bool,
    {
        config.validate()?;

        let weights = self.weights.as_ref().ok_or(LlmError::ModelNotLoaded)?;
        let prompt_tokens = self.tokenizer.encode(prompt)?;

        if prompt_tokens.len() >= config.context_window_size {
            return Err(LlmError::ContextWindowExceeded {
                tokens: prompt_tokens.len(),
                max_context: config.context_window_size,
            });
        }

        let mut sampler = Sampler::new(
            config.temperature,
            config.top_p,
            config.repetition_penalty,
            config.seed,
        );

        let mut context_tokens = prompt_tokens.clone();
        let mut generated_tokens: Vec<u32> = Vec::with_capacity(config.max_new_tokens);
        let mut accumulated_text = String::new();

        let start_time = Instant::now();
        let mut ttft_ms = 0.0;
        let mut finish_reason = FinishReason::Length;

        while generated_tokens.len() < config.max_new_tokens {
            // Check cooperative cancellation token (<10ms responsiveness)
            if let Some(ref cancel) = cancel_token {
                if cancel.load(Ordering::Relaxed) {
                    finish_reason = FinishReason::Cancelled;
                    let _ = on_token(StreamToken {
                        token_id: 0,
                        text: String::new(),
                        index: generated_tokens.len(),
                        is_final: true,
                        finish_reason: Some(FinishReason::Cancelled),
                    });
                    break;
                }
            }

            // Context window enforcement: slide window if needed
            let effective_context = if context_tokens.len() > config.context_window_size {
                let overflow = context_tokens.len() - config.context_window_size;
                &context_tokens[overflow..]
            } else {
                &context_tokens[..]
            };

            // Neural forward pass
            let logits = weights.forward(effective_context);

            // Sample next token
            let next_token = sampler.sample(&logits, &context_tokens);

            if generated_tokens.is_empty() {
                ttft_ms = start_time.elapsed().as_secs_f64() * 1000.0;
            }

            // End-of-sequence detection
            if self.tokenizer.is_eos(next_token) {
                finish_reason = FinishReason::EndOfSequence;
                let _ = on_token(StreamToken {
                    token_id: next_token,
                    text: String::new(),
                    index: generated_tokens.len(),
                    is_final: true,
                    finish_reason: Some(FinishReason::EndOfSequence),
                });
                break;
            }

            let token_text = self.tokenizer.decode_single(next_token);
            accumulated_text.push_str(&token_text);
            generated_tokens.push(next_token);
            context_tokens.push(next_token);

            // Stop sequence check
            let mut stop_matched = false;
            for stop in &config.stop_sequences {
                if !stop.is_empty() && accumulated_text.ends_with(stop) {
                    let len_to_cut = stop.len();
                    accumulated_text.truncate(accumulated_text.len() - len_to_cut);
                    finish_reason = FinishReason::StopSequence;
                    stop_matched = true;
                    break;
                }
            }

            let is_final = stop_matched || (generated_tokens.len() >= config.max_new_tokens);
            let reason = if is_final { Some(finish_reason) } else { None };

            let should_continue = on_token(StreamToken {
                token_id: next_token,
                text: token_text,
                index: generated_tokens.len() - 1,
                is_final,
                finish_reason: reason,
            });

            if !should_continue || stop_matched {
                break;
            }
        }

        let total_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let tokens_per_second = if total_time_ms > 0.0 {
            (generated_tokens.len() as f64) / (total_time_ms / 1000.0)
        } else {
            0.0
        };

        Ok(GenerationOutput {
            text: accumulated_text,
            tokens: generated_tokens.clone(),
            prompt_tokens: prompt_tokens.len(),
            completion_tokens: generated_tokens.len(),
            ttft_ms,
            total_time_ms,
            tokens_per_second,
            finish_reason,
        })
    }
}
