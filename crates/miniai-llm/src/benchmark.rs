//! Quantitative LLM Benchmark Suite.
//!
//! Provides comprehensive performance benchmarking for offline LLM models:
//! - Cold start model load time vs warm model verification.
//! - Time-To-First-Token (TTFT) in milliseconds.
//! - Token decoding throughput (tokens/second).
//! - Memory footprint tracking (bytes).
//! - Statistical aggregation: mean, median (p50), 95th percentile (p95).
//! - Structured JSON reporting conforming to MiniAI benchmark schemas.

use crate::config::GenerationConfig;
use crate::engine::LlmEngine;
use crate::error::LlmError;
use miniai_core::{AcceleratorBackend, Modality, VerificationTier};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Benchmark execution parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBenchmarkConfig {
    pub benchmark_id: String,
    pub model_id: String,
    pub model_path: PathBuf,
    pub backend: AcceleratorBackend,
    pub prompt: String,
    pub max_new_tokens: usize,
    pub warmup_iterations: usize,
    pub measured_iterations: usize,
    pub verification_tier: VerificationTier,
}

impl Default for LlmBenchmarkConfig {
    fn default() -> Self {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            benchmark_id: format!("bench-llm-{millis}"),
            model_id: "llama-3.2-1b-instruct-q4".to_string(),
            model_path: PathBuf::from("models/llama-3.2-1b.gguf"),
            backend: AcceleratorBackend::Cpu,
            prompt: "Explain the architectural principles of local-first multimodal computing.".to_string(),
            max_new_tokens: 32,
            warmup_iterations: 1,
            measured_iterations: 3,
            verification_tier: VerificationTier::ProvenOnHost,
        }
    }
}

/// Metrics recorded for a single inference iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIterationMetrics {
    pub iteration: usize,
    pub ttft_ms: f64,
    pub total_time_ms: f64,
    pub tokens_generated: usize,
    pub tokens_per_second: f64,
    pub memory_footprint_bytes: u64,
}

/// Aggregated statistical benchmark report matching MiniAI evaluation schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBenchmarkReport {
    pub benchmark_id: String,
    pub modality: Modality,
    pub adapter_id: String,
    pub model_id: String,
    pub backend: AcceleratorBackend,
    pub verification_tier: VerificationTier,
    pub cold_start_ms: f64,
    pub warm_load_ms: f64,
    pub prompt_tokens: usize,
    pub tokens_per_iteration: usize,
    pub warmup_iterations: usize,
    pub measured_iterations: usize,
    pub iterations: Vec<LlmIterationMetrics>,
    pub ttft_mean_ms: f64,
    pub ttft_p50_ms: f64,
    pub ttft_p95_ms: f64,
    pub throughput_mean_tok_per_sec: f64,
    pub throughput_p50_tok_per_sec: f64,
    pub throughput_p95_tok_per_sec: f64,
    pub peak_memory_bytes: u64,
    pub created_at_epoch_secs: u64,
}

impl LlmBenchmarkReport {
    /// Formats the report as pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Saves the report to a target JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self
            .to_json()
            .map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }
}

pub struct LlmBenchmarkHarness;

impl LlmBenchmarkHarness {
    /// Executes the benchmark suite on an active engine instance.
    pub fn run(
        config: &LlmBenchmarkConfig,
        engine: &mut LlmEngine,
    ) -> Result<LlmBenchmarkReport, LlmError> {
        if config.measured_iterations == 0 {
            return Err(LlmError::InvalidConfig {
                reason: "measured_iterations must be at least 1".to_string(),
            });
        }

        // 1. Measure Cold Start Load Time
        let cold_start_begin = Instant::now();
        engine.load_model(&config.model_id, &config.model_path, config.backend)?;
        let cold_start_ms = cold_start_begin.elapsed().as_secs_f64() * 1000.0;

        // 2. Measure Warm Memory Verification Time
        let warm_start_begin = Instant::now();
        let _ = engine.memory_footprint_bytes();
        let warm_load_ms = warm_start_begin.elapsed().as_secs_f64() * 1000.0;

        let gen_config = GenerationConfig {
            max_new_tokens: config.max_new_tokens,
            temperature: 0.7,
            top_p: 0.9,
            repetition_penalty: 1.1,
            seed: Some(42),
            ..Default::default()
        };

        let prompt_tokens_count = engine.tokenize(&config.prompt)?.len();

        // 3. Warmup Iterations (excluded from final statistics)
        for _ in 0..config.warmup_iterations {
            let _ = engine.generate(&config.prompt, &gen_config)?;
        }

        // 4. Measured Iterations
        let mut iteration_records: Vec<LlmIterationMetrics> =
            Vec::with_capacity(config.measured_iterations);
        let mut ttfts: Vec<f64> = Vec::with_capacity(config.measured_iterations);
        let mut throughputs: Vec<f64> = Vec::with_capacity(config.measured_iterations);
        let mut peak_mem = engine.memory_footprint_bytes();

        for i in 1..=config.measured_iterations {
            let output = engine.generate(&config.prompt, &gen_config)?;
            let mem = engine.memory_footprint_bytes();
            if mem > peak_mem {
                peak_mem = mem;
            }

            ttfts.push(output.ttft_ms);
            throughputs.push(output.tokens_per_second);

            iteration_records.push(LlmIterationMetrics {
                iteration: i,
                ttft_ms: output.ttft_ms,
                total_time_ms: output.total_time_ms,
                tokens_generated: output.completion_tokens,
                tokens_per_second: output.tokens_per_second,
                memory_footprint_bytes: mem,
            });
        }

        // 5. Statistical Calculations
        let ttft_mean = if ttfts.is_empty() {
            0.0
        } else {
            ttfts.iter().sum::<f64>() / ttfts.len() as f64
        };
        let throughput_mean = if throughputs.is_empty() {
            0.0
        } else {
            throughputs.iter().sum::<f64>() / throughputs.len() as f64
        };

        let ttft_p50 = percentile(&ttfts, 0.50);
        let ttft_p95 = percentile(&ttfts, 0.95);

        let throughput_p50 = percentile(&throughputs, 0.50);
        let throughput_p95 = percentile(&throughputs, 0.95);

        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(LlmBenchmarkReport {
            benchmark_id: config.benchmark_id.clone(),
            modality: Modality::Llm,
            adapter_id: format!("{}-{}-adapter", config.backend.display_name(), config.model_id),
            model_id: config.model_id.clone(),
            backend: config.backend,
            verification_tier: config.verification_tier,
            cold_start_ms,
            warm_load_ms,
            prompt_tokens: prompt_tokens_count,
            tokens_per_iteration: config.max_new_tokens,
            warmup_iterations: config.warmup_iterations,
            measured_iterations: config.measured_iterations,
            iterations: iteration_records,
            ttft_mean_ms: ttft_mean,
            ttft_p50_ms: ttft_p50,
            ttft_p95_ms: ttft_p95,
            throughput_mean_tok_per_sec: throughput_mean,
            throughput_p50_tok_per_sec: throughput_p50,
            throughput_p95_tok_per_sec: throughput_p95,
            peak_memory_bytes: peak_mem,
            created_at_epoch_secs: now_epoch,
        })
    }
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1);
    sorted[idx.min(sorted.len() - 1)]
}
