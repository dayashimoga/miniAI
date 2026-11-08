//! Real-model golden tests: load a genuine quantized GGUF and verify real neural behaviour.
//!
//! Requires weights, so the heavy tests are `#[ignore]`d by default. Run them with:
//!
//! ```text
//! MINIAI_TEST_GGUF=/path/to/SmolLM2-135M-Instruct-Q4_K_M.gguf \
//!   cargo test -p miniai-llm --test neural_real_model -- --ignored --nocapture
//! ```
//!
//! CI downloads the pinned SmolLM2-135M-Instruct Q4_K_M artifact and runs these tests.

use std::io::Write;
use std::path::PathBuf;

use miniai_llm::{
    ChatMessage, ChatTemplate, FinishReason, GenerationConfig, LlmError, NeuralEngine,
    PromptCompiler,
};

fn model_path() -> PathBuf {
    PathBuf::from(
        std::env::var("MINIAI_TEST_GGUF")
            .expect("set MINIAI_TEST_GGUF to a real .gguf file to run the real-model tests"),
    )
}

fn greedy(max_new_tokens: usize) -> GenerationConfig {
    GenerationConfig {
        max_new_tokens,
        context_window_size: 2048,
        temperature: 0.0,
        top_p: 1.0,
        repetition_penalty: 1.0,
        seed: Some(7),
        stop_sequences: vec![],
    }
}

fn chatml(question: &str) -> String {
    PromptCompiler::new(ChatTemplate::ChatMl).compile(&[ChatMessage::user(question)], None)
}

#[test]
fn bogus_gguf_is_rejected_not_faked() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(b"GGUF\x03\x00\x00\x00not-a-real-model")
        .unwrap();
    let err = NeuralEngine::load("bogus", f.path()).unwrap_err();
    assert!(matches!(err, LlmError::ModelLoadError { .. }), "{err:?}");

    let missing = NeuralEngine::load("missing", std::path::Path::new("/nonexistent/m.gguf"));
    assert!(matches!(missing, Err(LlmError::ModelLoadError { .. })));
}

#[test]
#[ignore = "requires real GGUF weights via MINIAI_TEST_GGUF"]
fn real_model_answers_questions_distinctly_and_coherently() {
    let mut engine = NeuralEngine::load("smollm2-135m-instruct-q4", &model_path()).unwrap();
    assert_eq!(engine.architecture(), "llama");

    let capital = engine
        .generate(
            &chatml("What is the capital of France? Answer in one word."),
            &greedy(24),
            None,
            |_| true,
        )
        .unwrap();
    println!(
        "capital => {:?} ({:.1} tok/s)",
        capital.text, capital.tokens_per_second
    );
    assert!(
        capital.text.to_lowercase().contains("paris"),
        "expected Paris, got: {}",
        capital.text
    );

    let adventure = engine
        .generate(&chatml("What is adventure?"), &greedy(48), None, |_| true)
        .unwrap();
    println!("adventure => {:?}", adventure.text);
    assert!(adventure.completion_tokens > 5);
    assert!(adventure.text.to_lowercase().contains("adventure") || adventure.text.len() > 40);
    // Different prompts must yield different completions (the original canned-response bug).
    assert_ne!(capital.text, adventure.text);

    // Real, measured telemetry
    assert!(adventure.prompt_eval_ms > 0.0);
    assert!(adventure.ttft_ms >= adventure.prompt_eval_ms);
    assert!(adventure.tokens_per_second > 0.0);
}

#[test]
#[ignore = "requires real GGUF weights via MINIAI_TEST_GGUF"]
fn real_model_is_deterministic_for_fixed_seed_and_streams_tokens() {
    let mut engine = NeuralEngine::load("smollm2-135m-instruct-q4", &model_path()).unwrap();
    let prompt = chatml("Name three primary colors.");

    let mut streamed = String::new();
    let a = engine
        .generate(&prompt, &greedy(32), None, |d| {
            streamed.push_str(d);
            true
        })
        .unwrap();
    let b = engine
        .generate(&prompt, &greedy(32), None, |_| true)
        .unwrap();

    assert_eq!(a.text, b.text, "greedy decoding must be reproducible");
    assert_eq!(
        streamed.trim_end(),
        a.text,
        "streamed deltas must reassemble the text"
    );
    assert!(matches!(
        a.finish_reason,
        FinishReason::Length | FinishReason::EndOfSequence
    ));
}

#[test]
#[ignore = "requires real GGUF weights via MINIAI_TEST_GGUF"]
fn real_model_honours_cancellation_and_context_limits() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let mut engine = NeuralEngine::load("smollm2-135m-instruct-q4", &model_path()).unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let c2 = cancel.clone();
    let mut seen = 0usize;
    let out = engine
        .generate(
            &chatml("Write a long story."),
            &greedy(200),
            Some(cancel),
            move |_| {
                seen += 1;
                if seen >= 3 {
                    c2.store(true, Ordering::Relaxed);
                }
                true
            },
        )
        .unwrap();
    assert_eq!(out.finish_reason, FinishReason::Cancelled);
    assert!(out.completion_tokens < 200);

    let huge = "word ".repeat(5000);
    let err = engine
        .generate(&chatml(&huge), &greedy(8), None, |_| true)
        .unwrap_err();
    assert!(matches!(err, LlmError::ContextWindowExceeded { .. }));
}
