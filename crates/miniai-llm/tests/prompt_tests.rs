use miniai_llm::{ChatMessage, ChatTemplate, PromptCompiler};

#[test]
fn test_llama3_prompt_compilation() {
    let compiler = PromptCompiler::new(ChatTemplate::Llama3);
    let messages = vec![
        ChatMessage::system("You are a helpful coding assistant."),
        ChatMessage::user("What is adventure?"),
    ];

    let compiled = compiler.compile(&messages, Some("Source: ARCHITECTURE.md#L10"));
    assert!(compiled.starts_with("<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n"));
    assert!(compiled.contains("You are a helpful coding assistant."));
    assert!(compiled.contains("[Grounding Sources / Citations]:\nSource: ARCHITECTURE.md#L10"));
    assert!(compiled
        .contains("<|start_header_id|>user<|end_header_id|>\n\nWhat is adventure?<|eot_id|>"));
    assert!(compiled.ends_with("<|start_header_id|>assistant<|end_header_id|>\n\n"));
}

#[test]
fn test_chatml_prompt_compilation() {
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let messages = vec![
        ChatMessage::user("Hello!"),
        ChatMessage::assistant("Hi there! How can I assist you?"),
        ChatMessage::user("Explain admission guard."),
    ];

    let compiled = compiler.compile(&messages, None);
    assert!(compiled.starts_with("<|im_start|>system\n"));
    assert!(compiled.contains("<|im_start|>user\nHello!<|im_end|>\n"));
    assert!(compiled.contains("<|im_start|>assistant\nHi there! How can I assist you?<|im_end|>\n"));
    assert!(compiled.ends_with("<|im_start|>assistant\n"));
}

#[test]
fn test_gemma_prompt_compilation() {
    let compiler = PromptCompiler::new(ChatTemplate::Gemma);
    let messages = vec![ChatMessage::user("Tell me about CPU baseline execution.")];

    let compiled = compiler.compile(&messages, Some("AdmissionGuard enforces 85% ceiling."));
    assert!(compiled.contains("<start_of_turn>user\n"));
    assert!(compiled.contains("AdmissionGuard enforces 85% ceiling."));
    assert!(compiled.contains("Tell me about CPU baseline execution."));
    assert!(compiled.ends_with("<start_of_turn>assistant\n"));
}

#[test]
fn test_sliding_window_truncation() {
    let compiler = PromptCompiler::default();
    let messages = vec![
        ChatMessage::user("Short message"),
        ChatMessage::assistant("Another response"),
        ChatMessage::user("Recent important question that must be preserved"),
    ];

    let truncated = compiler.truncate_sliding_window(&messages, 10);
    assert!(!truncated.is_empty());
    assert_eq!(
        truncated.last().unwrap().content,
        "Recent important question that must be preserved"
    );
}
