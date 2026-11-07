use miniai_llm::{AgentStep, ChatMessage, ChatTemplate, PromptCompiler, ReActAgent};
use miniai_rag::{ToolExecutionContext, ToolRegistry};
use std::sync::Arc;

#[test]
fn test_agent_system_prompt_generation() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler);

    let system_prompt = agent.build_system_prompt();
    assert!(system_prompt.contains("Thought:"));
    assert!(system_prompt.contains("Action:"));
    assert!(system_prompt.contains("Observation:"));
    assert!(system_prompt.contains("Final Answer:"));
    assert!(system_prompt.contains("calculator:"));
    assert!(system_prompt.contains("workspace_read_file:"));
    assert!(system_prompt.contains("code_symbol_search:"));
}

#[test]
fn test_agent_parse_thought_step() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler);

    let step = agent.parse_step("Thought: We should verify the local repository structure first.");
    assert_eq!(
        step,
        AgentStep::Thought("We should verify the local repository structure first.".to_string())
    );
}

#[test]
fn test_agent_parse_action_step() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler);

    let step = agent.parse_step(r#"Action: calculator({"expression": "10 * 5 + 3"})"#);
    assert_eq!(
        step,
        AgentStep::Action {
            tool: "calculator".to_string(),
            input: r#"{"expression": "10 * 5 + 3"}"#.to_string(),
        }
    );
}

#[test]
fn test_agent_parse_final_answer() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler);

    let step = agent.parse_step("Final Answer: 53 is the computed answer.");
    assert_eq!(
        step,
        AgentStep::FinalAnswer("53 is the computed answer.".to_string())
    );
}

#[test]
fn test_agent_execute_action_calculator() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler);

    let result = agent.execute_action("calculator", r#"{"expression": "12 * 12"}"#);
    assert!(result.success);
    assert_eq!(result.output, "144");
    assert!(result.error_message.is_none());
}

#[test]
fn test_agent_execute_action_unknown_tool() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler);

    let result = agent.execute_action("non_existent_tool", "{}");
    assert!(!result.success);
    assert!(result
        .error_message
        .unwrap()
        .contains("Tool 'non_existent_tool' not found"));
}

#[test]
fn test_agent_with_max_steps_and_compile_history() {
    let registry = Arc::new(ToolRegistry::new());
    let context = ToolExecutionContext::default();
    let compiler = PromptCompiler::new(ChatTemplate::ChatMl);
    let agent = ReActAgent::new(registry, context, compiler).with_max_steps(8);

    let history = vec![
        ChatMessage::user("What is 10 + 20?"),
        ChatMessage::assistant("Thought: Need to compute 10 + 20."),
    ];

    let prompt = agent.compile_prompt(&history);
    assert!(prompt.contains("<|im_start|>user\nWhat is 10 + 20?<|im_end|>"));
    assert!(prompt.contains("<|im_start|>assistant\nThought: Need to compute 10 + 20.<|im_end|>"));
}
