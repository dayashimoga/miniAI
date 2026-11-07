use serde::{Deserialize, Serialize};

/// Role of an conversational or tool execution turn within working memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A single turn in the working memory buffer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingTurn {
    pub role: TurnRole,
    pub content: String,
    pub estimated_tokens: usize,
    pub is_pinned: bool,
}

impl WorkingTurn {
    /// Creates a new working turn, estimating token count (approx 4 chars per token).
    pub fn new(role: TurnRole, content: impl Into<String>, is_pinned: bool) -> Self {
        let content = content.into();
        let estimated_tokens = content.len().div_ceil(4);
        Self {
            role,
            content,
            estimated_tokens,
            is_pinned,
        }
    }
}

/// Token-budgeted rolling context window for short-term working memory.
#[derive(Debug, Clone)]
pub struct WorkingMemory {
    max_tokens: usize,
    pinned_system_prompt: Option<WorkingTurn>,
    turns: Vec<WorkingTurn>,
    scratchpad: Vec<String>,
}

impl WorkingMemory {
    /// Creates a new `WorkingMemory` with the specified token capacity budget.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens: max_tokens.max(16),
            pinned_system_prompt: None,
            turns: Vec::new(),
            scratchpad: Vec::new(),
        }
    }

    /// Sets or replaces the pinned system instructions (never evicted).
    pub fn set_pinned_system_prompt(&mut self, prompt: &str) {
        self.pinned_system_prompt = Some(WorkingTurn::new(TurnRole::System, prompt, true));
        self.enforce_budget();
    }

    /// Appends a dialogue turn and evicts oldest unpinned turns if budget is exceeded.
    pub fn add_turn(&mut self, role: TurnRole, content: &str, is_pinned: bool) {
        let turn = WorkingTurn::new(role, content, is_pinned);
        self.turns.push(turn);
        self.enforce_budget();
    }

    /// Appends an active tool output or observation to the temporary scratchpad.
    pub fn add_scratchpad(&mut self, tool_name: &str, output: &str) {
        self.scratchpad
            .push(format!("[Tool Output: {tool_name}]\n{output}"));
    }

    /// Clears the scratchpad after an autonomous cycle concludes.
    pub fn clear_scratchpad(&mut self) {
        self.scratchpad.clear();
    }

    /// Returns current total tokens consumed across system prompt, turns, and scratchpad.
    pub fn total_tokens(&self) -> usize {
        let sys_tokens = self
            .pinned_system_prompt
            .as_ref()
            .map(|t| t.estimated_tokens)
            .unwrap_or(0);
        let turn_tokens: usize = self.turns.iter().map(|t| t.estimated_tokens).sum();
        let scratch_tokens: usize = self.scratchpad.iter().map(|s| s.len().div_ceil(4)).sum();
        sys_tokens + turn_tokens + scratch_tokens
    }

    /// Enforces the token budget by evicting oldest unpinned turns until total <= max_tokens.
    fn enforce_budget(&mut self) {
        while self.total_tokens() > self.max_tokens {
            // Find oldest unpinned turn index
            if let Some(idx) = self.turns.iter().position(|t| !t.is_pinned) {
                self.turns.remove(idx);
            } else {
                // All turns are pinned, cannot evict further
                break;
            }
        }
    }

    /// Returns all active turns formatted in chronological order.
    pub fn active_turns(&self) -> Vec<WorkingTurn> {
        let mut list = Vec::new();
        if let Some(sys) = &self.pinned_system_prompt {
            list.push(sys.clone());
        }
        list.extend(self.turns.clone());
        list
    }

    /// Returns scratchpad entries.
    pub fn scratchpad(&self) -> &[String] {
        &self.scratchpad
    }

    /// Clears all dialogue turns, preserving the pinned system prompt.
    pub fn clear_dialogue(&mut self) {
        self.turns.retain(|t| t.is_pinned);
        self.scratchpad.clear();
    }
}
