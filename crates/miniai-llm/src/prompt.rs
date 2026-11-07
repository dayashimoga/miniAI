use serde::{Deserialize, Serialize};

/// Supported chat instruction formatting templates for neural SLMs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChatTemplate {
    /// LLaMA-3 / LLaMA-3.2 instruction format using `<|begin_of_text|>`, `<|start_header_id|>`, etc.
    #[default]
    Llama3,
    /// OpenAI ChatML / Qwen / SmolLM format using `<|im_start|>` and `<|im_end|>`.
    ChatMl,
    /// Google Gemma format using `<start_of_turn>` and `<end_of_turn>`.
    Gemma,
    /// Raw unformatted prompt passing text directly.
    Raw,
}

/// Message role in a multi-turn conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

/// A single message in a multi-turn conversation context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }

    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
        }
    }
}

/// The Prompt Compiler formats multi-turn chat messages, system instructions,
/// and retrieved RAG context into model-native instruction strings.
#[derive(Debug, Clone)]
pub struct PromptCompiler {
    pub template: ChatTemplate,
    pub default_system_prompt: Option<String>,
    pub max_context_tokens: usize,
}

impl Default for PromptCompiler {
    fn default() -> Self {
        Self {
            template: ChatTemplate::Llama3,
            default_system_prompt: Some(
                "You are MiniAI, a production-grade, local-first multimodal assistant executing offline with strict privacy guarantees."
                    .to_string(),
            ),
            max_context_tokens: 4096,
        }
    }
}

impl PromptCompiler {
    /// Creates a new `PromptCompiler` for the given template.
    pub fn new(template: ChatTemplate) -> Self {
        Self {
            template,
            ..Default::default()
        }
    }

    /// Compiles a sequence of chat messages into a model-native formatted string.
    pub fn compile(&self, messages: &[ChatMessage], rag_grounding: Option<&str>) -> String {
        match self.template {
            ChatTemplate::Llama3 => self.compile_llama3(messages, rag_grounding),
            ChatTemplate::ChatMl => self.compile_chatml(messages, rag_grounding),
            ChatTemplate::Gemma => self.compile_gemma(messages, rag_grounding),
            ChatTemplate::Raw => self.compile_raw(messages, rag_grounding),
        }
    }

    fn compile_llama3(&self, messages: &[ChatMessage], rag_grounding: Option<&str>) -> String {
        let mut out = String::from("<|begin_of_text|>");

        // System message
        let sys_content = messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| m.content.as_str())
            .or(self.default_system_prompt.as_deref());

        if let Some(sys) = sys_content {
            out.push_str("<|start_header_id|>system<|end_header_id|>\n\n");
            out.push_str(sys);
            if let Some(rag) = rag_grounding {
                out.push_str("\n\n[Grounding Sources / Citations]:\n");
                out.push_str(rag);
            }
            out.push_str("<|eot_id|>");
        } else if let Some(rag) = rag_grounding {
            out.push_str("<|start_header_id|>system<|end_header_id|>\n\n[Grounding Sources]:\n");
            out.push_str(rag);
            out.push_str("<|eot_id|>");
        }

        for msg in messages {
            match msg.role {
                MessageRole::System => continue, // already handled
                MessageRole::User => {
                    out.push_str("<|start_header_id|>user<|end_header_id|>\n\n");
                    out.push_str(&msg.content);
                    out.push_str("<|eot_id|>");
                }
                MessageRole::Assistant => {
                    out.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
                    out.push_str(&msg.content);
                    out.push_str("<|eot_id|>");
                }
                MessageRole::Tool => {
                    out.push_str("<|start_header_id|>ipython<|end_header_id|>\n\n");
                    out.push_str(&msg.content);
                    out.push_str("<|eot_id|>");
                }
            }
        }

        // Prompt end to invite assistant completion
        out.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
        out
    }

    fn compile_chatml(&self, messages: &[ChatMessage], rag_grounding: Option<&str>) -> String {
        let mut out = String::new();

        let sys_content = messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| m.content.as_str())
            .or(self.default_system_prompt.as_deref());

        if let Some(sys) = sys_content {
            out.push_str("<|im_start|>system\n");
            out.push_str(sys);
            if let Some(rag) = rag_grounding {
                out.push_str("\n\nContext:\n");
                out.push_str(rag);
            }
            out.push_str("<|im_end|>\n");
        }

        for msg in messages {
            match msg.role {
                MessageRole::System => continue,
                MessageRole::User => {
                    out.push_str("<|im_start|>user\n");
                    out.push_str(&msg.content);
                    out.push_str("<|im_end|>\n");
                }
                MessageRole::Assistant => {
                    out.push_str("<|im_start|>assistant\n");
                    out.push_str(&msg.content);
                    out.push_str("<|im_end|>\n");
                }
                MessageRole::Tool => {
                    out.push_str("<|im_start|>tool\n");
                    out.push_str(&msg.content);
                    out.push_str("<|im_end|>\n");
                }
            }
        }

        out.push_str("<|im_start|>assistant\n");
        out
    }

    fn compile_gemma(&self, messages: &[ChatMessage], rag_grounding: Option<&str>) -> String {
        let mut out = String::new();

        let sys = self.default_system_prompt.as_deref().unwrap_or("");
        let mut user_prefix = String::new();
        if !sys.is_empty() {
            user_prefix.push_str(sys);
            user_prefix.push_str("\n\n");
        }
        if let Some(rag) = rag_grounding {
            user_prefix.push_str("Grounded Context:\n");
            user_prefix.push_str(rag);
            user_prefix.push_str("\n\n");
        }

        let mut is_first_user = true;
        for msg in messages {
            match msg.role {
                MessageRole::System => continue,
                MessageRole::User => {
                    out.push_str("<start_of_turn>user\n");
                    if is_first_user && !user_prefix.is_empty() {
                        out.push_str(&user_prefix);
                        is_first_user = false;
                    }
                    out.push_str(&msg.content);
                    out.push_str("<end_of_turn>\n");
                }
                MessageRole::Assistant | MessageRole::Tool => {
                    out.push_str("<start_of_turn>assistant\n");
                    out.push_str(&msg.content);
                    out.push_str("<end_of_turn>\n");
                }
            }
        }

        out.push_str("<start_of_turn>assistant\n");
        out
    }

    fn compile_raw(&self, messages: &[ChatMessage], rag_grounding: Option<&str>) -> String {
        let mut out = String::new();
        if let Some(rag) = rag_grounding {
            out.push_str("Context:\n");
            out.push_str(rag);
            out.push_str("\n\n");
        }
        for msg in messages {
            out.push_str(&msg.content);
            out.push('\n');
        }
        out
    }

    /// Truncates conversational history using a sliding window to fit within token budgets.
    pub fn truncate_sliding_window(
        &self,
        messages: &[ChatMessage],
        max_words: usize,
    ) -> Vec<ChatMessage> {
        let mut total_words = 0;
        let mut kept = Vec::new();

        // Always prioritize the most recent messages while retaining the system prompt
        for msg in messages.iter().rev() {
            let words = msg.content.split_whitespace().count();
            if total_words + words <= max_words || kept.is_empty() {
                total_words += words;
                kept.push(msg.clone());
            } else {
                break;
            }
        }

        kept.reverse();
        kept
    }
}
