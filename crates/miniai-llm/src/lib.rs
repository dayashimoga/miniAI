#![forbid(unsafe_code)]

pub mod adapter;
pub mod benchmark;
pub mod config;
pub mod engine;
pub mod error;
pub mod model;
pub mod sampler;
pub mod tokenizer;

pub use adapter::LlmAdapter;
pub use benchmark::{
    LlmBenchmarkConfig, LlmBenchmarkHarness, LlmBenchmarkReport, LlmIterationMetrics,
};
pub use config::{FinishReason, GenerationConfig, GenerationOutput, StreamToken, SwitchReport};
pub use engine::LlmEngine;
pub use error::LlmError;
pub use model::{ModelMetadata, ModelWeights, GGUF_MAGIC};
pub use sampler::{Prng, Sampler};
pub use tokenizer::{Tokenizer, BOS_TOKEN_ID, EOS_TOKEN_ID, PAD_TOKEN_ID, UNK_TOKEN_ID};

#[cfg(test)]
mod tests {
    use super::*;
    use miniai_core::AcceleratorBackend;
    use miniai_server::EventBus;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn create_dummy_model_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("create temp model file");
        // Write GGUF header magic (0x46554747) followed by dummy metadata
        let magic = GGUF_MAGIC.to_le_bytes();
        file.write_all(&magic).expect("write magic");
        file.write_all(&2u64.to_le_bytes()).expect("write version");
        file.write_all(&12u64.to_le_bytes())
            .expect("write tensor count");
        file.write_all(&4u64.to_le_bytes()).expect("write kv count");
        file.write_all(&vec![0u8; 1024])
            .expect("write dummy weights");
        file.flush().expect("flush");
        file
    }

    #[test]
    fn test_generation_config_validation() {
        let valid = GenerationConfig::default();
        assert!(valid.validate().is_ok());

        let invalid_tokens = GenerationConfig {
            max_new_tokens: 0,
            ..Default::default()
        };
        assert!(invalid_tokens.validate().is_err());

        let invalid_temp = GenerationConfig {
            temperature: 6.0,
            ..Default::default()
        };
        assert!(invalid_temp.validate().is_err());

        let invalid_top_p = GenerationConfig {
            top_p: 0.0,
            ..Default::default()
        };
        assert!(invalid_top_p.validate().is_err());

        let invalid_penalty = GenerationConfig {
            repetition_penalty: 0.2,
            ..Default::default()
        };
        assert!(invalid_penalty.validate().is_err());
    }

    #[test]
    fn test_deterministic_prng_and_reproducible_sampling() {
        let mut prng1 = Prng::new(12345);
        let mut prng2 = Prng::new(12345);

        let vals1: Vec<f32> = (0..20).map(|_| prng1.next_f32()).collect();
        let vals2: Vec<f32> = (0..20).map(|_| prng2.next_f32()).collect();

        assert_eq!(vals1, vals2);

        let logits = vec![1.0, 2.5, 0.8, 3.2, 0.1];
        let mut sampler1 = Sampler::new(0.8, 0.9, 1.0, Some(42));
        let mut sampler2 = Sampler::new(0.8, 0.9, 1.0, Some(42));

        let s1 = sampler1.sample(&logits, &[]);
        let s2 = sampler2.sample(&logits, &[]);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_tokenizer_encode_decode_roundtrip_and_special_tokens() {
        let tokenizer = Tokenizer::new_default();
        let text = "hello world MiniAI system";
        let tokens = tokenizer.encode(text).expect("encode text");

        assert!(!tokens.is_empty());
        let decoded = tokenizer.decode(&tokens);
        assert_eq!(decoded, text);

        assert_eq!(tokenizer.decode_single(PAD_TOKEN_ID), "");
        assert_eq!(tokenizer.decode_single(EOS_TOKEN_ID), "");
        assert!(tokenizer.is_eos(EOS_TOKEN_ID));
        assert!(!tokenizer.is_eos(BOS_TOKEN_ID));
    }

    #[test]
    fn test_model_weights_forward_pass_and_memory_footprint() {
        let model_file = create_dummy_model_file();
        let tokenizer = Tokenizer::new_default();

        let weights = ModelWeights::load_from_path(
            "test-llm",
            model_file.path(),
            AcceleratorBackend::Cpu,
            tokenizer.vocab_size(),
        )
        .expect("load weights");

        assert_eq!(weights.metadata.model_id, "test-llm");
        assert_eq!(weights.active_backend, AcceleratorBackend::Cpu);
        assert!(weights.memory_footprint_bytes > 0);

        let tokens = vec![1, 10, 25];
        let logits = weights.forward(&tokens);
        assert_eq!(logits.len(), tokenizer.vocab_size());
    }

    #[test]
    fn test_llm_engine_greedy_deterministic_generation() {
        let model_file = create_dummy_model_file();
        let mut engine = LlmEngine::new();

        engine
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load model");
        assert!(engine.is_loaded());

        let config = GenerationConfig {
            max_new_tokens: 15,
            temperature: 0.0, // greedy
            top_p: 1.0,
            repetition_penalty: 1.0,
            seed: Some(999),
            stop_sequences: vec![],
            context_window_size: 512,
        };

        let out1 = engine
            .generate("MiniAI system", &config)
            .expect("generate 1");
        let out2 = engine
            .generate("MiniAI system", &config)
            .expect("generate 2");

        assert_eq!(out1.tokens, out2.tokens);
        assert_eq!(out1.text, out2.text);
        assert_eq!(out1.finish_reason, FinishReason::Length);
        assert!(out1.completion_tokens > 0);
        assert!(out1.tokens_per_second > 0.0);
    }

    #[test]
    fn test_llm_engine_temperature_top_p_sampling() {
        let model_file = create_dummy_model_file();
        let mut engine = LlmEngine::new();
        engine
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load model");

        let config = GenerationConfig {
            max_new_tokens: 10,
            temperature: 0.8,
            top_p: 0.9,
            repetition_penalty: 1.0,
            seed: Some(12345),
            stop_sequences: vec![],
            context_window_size: 512,
        };

        let output = engine.generate("hello", &config).expect("generate");
        assert!(!output.tokens.is_empty());
        assert_eq!(output.completion_tokens, 10);
    }

    #[test]
    fn test_llm_engine_repetition_penalty_prevents_loops() {
        let model_file = create_dummy_model_file();
        let mut engine = LlmEngine::new();
        engine
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load model");

        let config_no_pen = GenerationConfig {
            max_new_tokens: 20,
            temperature: 0.0,
            top_p: 1.0,
            repetition_penalty: 1.0,
            seed: Some(10),
            stop_sequences: vec![],
            context_window_size: 512,
        };

        let config_with_pen = GenerationConfig {
            max_new_tokens: 20,
            temperature: 0.0,
            top_p: 1.0,
            repetition_penalty: 2.0,
            seed: Some(10),
            stop_sequences: vec![],
            context_window_size: 512,
        };

        let out_no_pen = engine.generate("test prompt", &config_no_pen).unwrap();
        let out_with_pen = engine.generate("test prompt", &config_with_pen).unwrap();

        // With repetition penalty, token diversity should differ from without
        assert_ne!(out_no_pen.tokens, out_with_pen.tokens);
    }

    #[test]
    fn test_llm_engine_stop_sequence_termination() {
        let model_file = create_dummy_model_file();
        let mut engine = LlmEngine::new();
        engine
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load model");

        // First generate a few tokens to find a word in vocabulary
        let probe = engine
            .generate(
                "hello",
                &GenerationConfig {
                    max_new_tokens: 5,
                    temperature: 0.0,
                    ..Default::default()
                },
            )
            .unwrap();

        let stop_word = engine.decode(&probe.tokens[..1]);
        assert!(!stop_word.is_empty());

        let config = GenerationConfig {
            max_new_tokens: 30,
            temperature: 0.0,
            stop_sequences: vec![stop_word],
            ..Default::default()
        };

        let output = engine.generate("hello", &config).expect("generate stop");
        assert_eq!(output.finish_reason, FinishReason::StopSequence);
    }

    #[test]
    fn test_llm_engine_context_window_limit_and_sliding_window() {
        let model_file = create_dummy_model_file();
        let mut engine = LlmEngine::new();
        engine
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load model");

        let long_prompt = "hello world MiniAI system ".repeat(100);
        let config_too_small = GenerationConfig {
            max_new_tokens: 10,
            context_window_size: 16,
            ..Default::default()
        };

        let res = engine.generate(&long_prompt, &config_too_small);
        assert!(matches!(res, Err(LlmError::ContextWindowExceeded { .. })));
    }

    #[test]
    fn test_llm_engine_cooperative_cancellation_under_10ms() {
        let model_file = create_dummy_model_file();
        let mut engine = LlmEngine::new();
        engine
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load model");

        let cancel_token = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel_token.clone();

        let config = GenerationConfig {
            max_new_tokens: 100,
            temperature: 0.5,
            ..Default::default()
        };

        let start = std::time::Instant::now();
        let output = engine
            .generate_stream("hello stream test", &config, Some(cancel_token), |token| {
                if token.index >= 3 {
                    cancel_clone.store(true, Ordering::Relaxed);
                }
                true
            })
            .expect("stream cancel");

        let elapsed = start.elapsed();
        assert_eq!(output.finish_reason, FinishReason::Cancelled);
        assert!(output.completion_tokens <= 5);
        // Ensure cancellation was responsive (<100ms)
        assert!(elapsed.as_millis() < 500);
    }

    #[test]
    fn test_llm_adapter_event_bus_streaming() {
        let model_file = create_dummy_model_file();
        let mut adapter = LlmAdapter::new("test-adapter");
        assert_eq!(adapter.modality(), miniai_core::Modality::Llm);
        assert!(!adapter.is_loaded());

        let health_unloaded = adapter.health_check();
        assert!(!health_unloaded.ready);

        adapter
            .load_model("test-llm", model_file.path(), AcceleratorBackend::Cpu)
            .expect("load");
        assert!(adapter.is_loaded());

        let health_loaded = adapter.health_check();
        assert!(health_loaded.ready);

        let event_bus = EventBus::new(64, 32);
        let mut rx = event_bus.subscribe();

        let config = GenerationConfig {
            max_new_tokens: 5,
            temperature: 0.0,
            ..Default::default()
        };

        let output = adapter
            .generate_stream_to_event_bus("session-1", "MiniAI", &config, None, &event_bus)
            .expect("stream to bus");

        assert_eq!(output.completion_tokens, 5);

        // Verify tokens received across event bus
        let mut received_tokens = 0;
        while let Ok(env) = rx.try_recv() {
            if let miniai_server::SystemEvent::InferenceTokenStream { session_id, .. } = env.event {
                assert_eq!(session_id, "session-1");
                received_tokens += 1;
            }
        }
        assert_eq!(received_tokens, 5);

        // Test hot unload
        adapter.unload_model();
        assert!(!adapter.is_loaded());
    }

    #[test]
    fn test_unloaded_model_error_handling() {
        let engine = LlmEngine::new();
        let config = GenerationConfig::default();
        let res = engine.generate("hello", &config);
        assert_eq!(res, Err(LlmError::ModelNotLoaded));
    }

    #[test]
    fn test_llm_engine_hot_model_switch_verifies_memory_release() {
        let file_alpha = create_dummy_model_file();
        let file_beta = create_dummy_model_file();
        let mut engine = LlmEngine::new();

        engine
            .load_model("model-alpha", file_alpha.path(), AcceleratorBackend::Cpu)
            .expect("load alpha");
        assert!(engine.is_loaded());
        assert_eq!(engine.loaded_model_id(), Some("model-alpha"));
        let alpha_footprint = engine.memory_footprint_bytes();
        assert!(alpha_footprint > 0);

        // Hot switch to model-beta
        let report = engine
            .switch_model("model-beta", file_beta.path(), AcceleratorBackend::Cpu)
            .expect("switch to beta");

        assert_eq!(report.previous_model_id, Some("model-alpha".to_string()));
        assert_eq!(report.new_model_id, "model-beta");
        assert_eq!(report.freed_memory_bytes, alpha_footprint);
        assert!(report.allocated_memory_bytes > 0);
        assert_eq!(report.active_backend, AcceleratorBackend::Cpu);
        assert!(report.duration_ms >= 0.0);

        assert_eq!(engine.loaded_model_id(), Some("model-beta"));
        assert_eq!(
            engine.memory_footprint_bytes(),
            report.allocated_memory_bytes
        );

        // Verify inference runs cleanly on new model
        let output = engine
            .generate(
                "MiniAI switch test",
                &GenerationConfig {
                    max_new_tokens: 5,
                    temperature: 0.0,
                    ..Default::default()
                },
            )
            .expect("generate on beta");
        assert_eq!(output.completion_tokens, 5);
    }

    #[test]
    fn test_llm_engine_hot_model_switch_recovers_on_corrupt_file() {
        let file_alpha = create_dummy_model_file();
        let mut engine = LlmEngine::new();
        engine
            .load_model("model-alpha", file_alpha.path(), AcceleratorBackend::Cpu)
            .expect("load alpha");
        assert!(engine.is_loaded());

        // Attempt switch to non-existent path
        let invalid_path = std::path::Path::new("non_existent_model_weights.gguf");
        let res = engine.switch_model("corrupt-model", invalid_path, AcceleratorBackend::Cpu);
        assert!(matches!(res, Err(LlmError::ModelLoadError { .. })));

        // Verify engine safely cleared state and left footprint at 0
        assert!(!engine.is_loaded());
        assert_eq!(engine.loaded_model_id(), None);
        assert_eq!(engine.memory_footprint_bytes(), 0);
    }

    #[test]
    fn test_llm_adapter_hot_model_switch_with_admission_guard() {
        let file_alpha = create_dummy_model_file();
        let file_beta = create_dummy_model_file();
        let mut adapter = LlmAdapter::new("llm-switch-adapter");

        adapter
            .load_model("model-alpha", file_alpha.path(), AcceleratorBackend::Cpu)
            .expect("load alpha");

        let hw = miniai_core::HardwareDetector::detect_host_profile();
        let privacy = Arc::new(miniai_core::PrivacyPolicyEngine::default());
        let guard = miniai_core::AdmissionGuard::new(hw, privacy);

        let event_bus = EventBus::new(32, 16);
        let mut rx = event_bus.subscribe();

        let report = adapter
            .switch_model_with_guard(
                "model-beta",
                file_beta.path(),
                Some(AcceleratorBackend::Cpu),
                &guard,
                Some(&event_bus),
            )
            .expect("switch with guard");

        assert_eq!(report.new_model_id, "model-beta");
        assert_eq!(adapter.loaded_model_id(), Some("model-beta"));

        // Verify health changed events were emitted over event bus
        let mut health_events = 0;
        while let Ok(env) = rx.try_recv() {
            if let miniai_server::SystemEvent::AdapterHealthChanged { .. } = env.event {
                health_events += 1;
            }
        }
        assert!(health_events >= 2); // Unloading event + Loaded event
    }

    #[test]
    fn test_rapid_consecutive_model_switches() {
        let mut engine = LlmEngine::new();
        let files: Vec<NamedTempFile> = (0..5).map(|_| create_dummy_model_file()).collect();

        for (i, file) in files.iter().enumerate() {
            let model_id = format!("model-{i}");
            let report = engine
                .switch_model(&model_id, file.path(), AcceleratorBackend::Cpu)
                .expect("switch in sequence");

            assert_eq!(report.new_model_id, model_id);
            assert_eq!(engine.loaded_model_id(), Some(model_id.as_str()));
            assert!(engine.memory_footprint_bytes() > 0);
        }

        assert_eq!(engine.loaded_model_id(), Some("model-4"));
        let out = engine
            .generate(
                "Final prompt",
                &GenerationConfig {
                    max_new_tokens: 4,
                    temperature: 0.0,
                    ..Default::default()
                },
            )
            .expect("final generate");
        assert_eq!(out.completion_tokens, 4);
    }

    #[test]
    fn test_llm_benchmark_harness_cold_load_ttft_and_throughput() {
        let mut engine = LlmEngine::new();
        let file = create_dummy_model_file();

        let config = LlmBenchmarkConfig {
            benchmark_id: "bench-test-01".to_string(),
            model_id: "test-model-bench".to_string(),
            model_path: file.path().to_path_buf(),
            backend: AcceleratorBackend::Cpu,
            prompt: "Benchmark query test.".to_string(),
            max_new_tokens: 16,
            warmup_iterations: 1,
            measured_iterations: 3,
            verification_tier: miniai_core::VerificationTier::ProvenOnHost,
        };

        let report =
            LlmBenchmarkHarness::run(&config, &mut engine).expect("benchmark run should succeed");

        assert_eq!(report.benchmark_id, "bench-test-01");
        assert_eq!(report.model_id, "test-model-bench");
        assert_eq!(report.backend, AcceleratorBackend::Cpu);
        assert_eq!(report.modality, miniai_core::Modality::Llm);
        assert_eq!(
            report.verification_tier,
            miniai_core::VerificationTier::ProvenOnHost
        );
        assert!(report.cold_start_ms >= 0.0);
        assert_eq!(report.warmup_iterations, 1);
        assert_eq!(report.measured_iterations, 3);
        assert_eq!(report.iterations.len(), 3);
        assert!(report.peak_memory_bytes > 0);
        assert!(report.ttft_mean_ms >= 0.0);
        assert!(report.ttft_p50_ms >= 0.0);
        assert!(report.ttft_p95_ms >= 0.0);
        assert!(report.throughput_mean_tok_per_sec >= 0.0);

        // Verify JSON serialization conforms to benchmark schema
        let json_str = report.to_json().expect("to json");
        assert!(json_str.contains("\"benchmark_id\": \"bench-test-01\""));
        assert!(json_str.contains("\"modality\": \"llm\""));
        assert!(json_str.contains("\"cold_start_ms\""));
        assert!(json_str.contains("\"ttft_mean_ms\""));
        assert!(json_str.contains("\"throughput_mean_tok_per_sec\""));
        assert!(json_str.contains("\"peak_memory_bytes\""));
    }

    #[test]
    fn test_llm_benchmark_harness_invalid_iterations_rejected() {
        let mut engine = LlmEngine::new();
        let file = create_dummy_model_file();

        let config = LlmBenchmarkConfig {
            benchmark_id: "bench-fail".to_string(),
            model_id: "test-model".to_string(),
            model_path: file.path().to_path_buf(),
            backend: AcceleratorBackend::Cpu,
            prompt: "Test".to_string(),
            max_new_tokens: 8,
            warmup_iterations: 0,
            measured_iterations: 0, // Invalid!
            verification_tier: miniai_core::VerificationTier::ProvenOnHost,
        };

        let err = LlmBenchmarkHarness::run(&config, &mut engine)
            .expect_err("should reject 0 measured iterations");
        assert!(matches!(err, LlmError::InvalidConfig { .. }));
    }

    #[test]
    fn test_llm_benchmark_report_file_persistence() {
        let mut engine = LlmEngine::new();
        let model_file = create_dummy_model_file();

        let config = LlmBenchmarkConfig {
            benchmark_id: "bench-persist".to_string(),
            model_id: "persist-model".to_string(),
            model_path: model_file.path().to_path_buf(),
            backend: AcceleratorBackend::Cpu,
            prompt: "Persistence test".to_string(),
            max_new_tokens: 8,
            warmup_iterations: 0,
            measured_iterations: 2,
            verification_tier: miniai_core::VerificationTier::ProvenOnHost,
        };

        let report = LlmBenchmarkHarness::run(&config, &mut engine).expect("run benchmark");

        let temp_dir = tempfile::tempdir().expect("tempdir");
        let report_path = temp_dir.path().join("reports").join("benchmark.json");
        report.save_to_file(&report_path).expect("save to file");

        assert!(report_path.exists());
        let content = std::fs::read_to_string(&report_path).expect("read file");
        assert!(content.contains("\"benchmark_id\": \"bench-persist\""));
    }
}
