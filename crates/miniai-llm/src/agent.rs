use serde::{Deserialize, Serialize};
use std::sync::Arc;

use miniai_rag::tools::{ToolExecutionContext, ToolExecutionResult, ToolRegistry};

use crate::prompt::{ChatMessage, PromptCompiler};

/// State of an autonomous agent execution step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum AgentStep {
    /// Agent internal reasoning trace.
    Thought(String),
    /// Agent action invocation intent.
    Action { tool: String, input: String },
    /// Observation resulting from tool execution.
    Observation(String),
    /// Final concluded answer for the user.
    FinalAnswer(String),
}

/// Autonomous ReAct (Reasoning + Acting) Agent orchestrator.
pub struct ReActAgent {
    tool_registry: Arc<ToolRegistry>,
    context: ToolExecutionContext,
    compiler: PromptCompiler,
    max_steps: usize,
}

impl ReActAgent {
    /// Creates a new `ReActAgent` with registered tools and execution context.
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        context: ToolExecutionContext,
        compiler: PromptCompiler,
    ) -> Self {
        Self {
            tool_registry,
            context,
            compiler,
            max_steps: 5,
        }
    }

    /// Sets the maximum allowable tool reasoning steps before forcing conclusion.
    pub fn with_max_steps(mut self, steps: usize) -> Self {
        self.max_steps = steps.max(1);
        self
    }

    /// Builds the system prompt injecting tool definitions and ReAct formatting guidelines.
    pub fn build_system_prompt(&self) -> String {
        let tools = self.tool_registry.describe_tools();
        let mut prompt = String::from(
            "You are an autonomous AI agent capable of using local sandboxed tools to answer user questions.\n\
             Answer the user's request using the following format:\n\n\
             Thought: Consider what action to take next.\n\
             Action: tool_name({\"param\": \"value\"})\n\
             Observation: result of the tool.\n\
             ... (repeat Thought/Action/Observation as needed)\n\
             Final Answer: The final response to the user.\n\n\
             Available Tools:\n",
        );

        for t in tools {
            prompt.push_str(&format!("- {}: {}\n", t.name, t.description));
        }

        prompt
    }

    /// Parses an assistant output chunk into an `AgentStep`.
    pub fn parse_step(&self, raw_output: &str) -> AgentStep {
        let trimmed = raw_output.trim();

        if let Some(pos) = trimmed.find("Final Answer:") {
            let answer = trimmed[pos + "Final Answer:".len()..].trim();
            return AgentStep::FinalAnswer(answer.to_string());
        }

        if let Some(action_pos) = trimmed.find("Action:") {
            let action_str = trimmed[action_pos + "Action:".len()..].trim();
            if let Some(paren_open) = action_str.find('(') {
                let tool_name = action_str[..paren_open].trim();
                let input = if let Some(paren_close) = action_str.rfind(')') {
                    action_str[paren_open + 1..paren_close].trim()
                } else {
                    action_str[paren_open + 1..].trim()
                };
                return AgentStep::Action {
                    tool: tool_name.to_string(),
                    input: input.to_string(),
                };
            }
        }

        if let Some(thought_pos) = trimmed.find("Thought:") {
            let thought_str = trimmed[thought_pos + "Thought:".len()..].trim();
            return AgentStep::Thought(thought_str.to_string());
        }

        AgentStep::FinalAnswer(trimmed.to_string())
    }

    /// Safely executes an action using the sandboxed tool registry.
    pub fn execute_action(&self, tool: &str, input_json: &str) -> ToolExecutionResult {
        self.tool_registry
            .execute_tool(tool, input_json, &self.context)
    }

    /// Compiles conversational history with tool observations for the next iteration.
    pub fn compile_prompt(&self, history: &[ChatMessage]) -> String {
        self.compiler.compile(history, None)
    }
}
