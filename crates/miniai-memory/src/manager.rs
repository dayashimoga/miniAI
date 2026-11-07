use miniai_storage::StorageEngine;
use std::sync::{Arc, Mutex};

use crate::episodic::EpisodicMemoryService;
use crate::error::MemoryError;
use crate::semantic::SemanticMemoryService;
use crate::working::{TurnRole, WorkingMemory, WorkingTurn};

/// Unified coordinator managing all three memory tiers:
/// 1. Working Memory (Short-Term token-budgeted rolling context)
/// 2. Episodic Memory (Cross-session conversation summaries)
/// 3. Semantic Memory (Durable extracted facts & preferences)
pub struct ThreeTierMemoryManager {
    working: Mutex<WorkingMemory>,
    episodic: EpisodicMemoryService,
    semantic: SemanticMemoryService,
}

impl ThreeTierMemoryManager {
    /// Initializes the memory coordinator with persistent storage and working token capacity.
    pub fn new(storage: Arc<StorageEngine>, working_token_budget: usize) -> Self {
        Self {
            working: Mutex::new(WorkingMemory::new(working_token_budget)),
            episodic: EpisodicMemoryService::new(storage.clone()),
            semantic: SemanticMemoryService::new(storage),
        }
    }

    /// Access the working memory tier.
    pub fn working_memory(&self) -> &Mutex<WorkingMemory> {
        &self.working
    }

    /// Access the episodic memory service tier.
    pub fn episodic_service(&self) -> &EpisodicMemoryService {
        &self.episodic
    }

    /// Access the semantic memory service tier.
    pub fn semantic_service(&self) -> &SemanticMemoryService {
        &self.semantic
    }

    /// Assembles an augmented memory context block incorporating relevant semantic facts
    /// and recent episodic history for LLM generation.
    pub fn assemble_memory_context(&self, query: &str) -> Result<String, MemoryError> {
        let mut context_block = String::new();

        // 1. Retrieve relevant semantic facts
        let relevant_facts = self.semantic.search(None, Some(query))?;
        let durable_preferences = self.semantic.search(Some("preference"), None)?;

        let mut fact_lines = Vec::new();
        for fact in relevant_facts.iter().chain(durable_preferences.iter()) {
            fact_lines.push(format!("- [{}] {}", fact.category, fact.content));
        }
        fact_lines.sort();
        fact_lines.dedup();

        if !fact_lines.is_empty() {
            context_block.push_str("### Known User Facts & Preferences:\n");
            for line in fact_lines {
                context_block.push_str(&line);
                context_block.push('\n');
            }
            context_block.push('\n');
        }

        // 2. Retrieve relevant episodic summaries
        let recent_episodes = self.episodic.list_recent(2)?;
        if !recent_episodes.is_empty() {
            context_block.push_str("### Prior Conversation Context:\n");
            for ep in recent_episodes {
                context_block.push_str(&format!("- [{}] {}\n", ep.topic, ep.summary));
            }
            context_block.push('\n');
        }

        Ok(context_block)
    }

    /// Appends a new turn to working memory.
    pub fn record_turn(
        &self,
        role: TurnRole,
        content: &str,
        is_pinned: bool,
    ) -> Result<(), MemoryError> {
        let mut working = self
            .working
            .lock()
            .map_err(|_| MemoryError::Serialization("Working memory lock poisoned".to_string()))?;
        working.add_turn(role, content, is_pinned);
        Ok(())
    }

    /// Returns the active working memory turns.
    pub fn active_working_turns(&self) -> Result<Vec<WorkingTurn>, MemoryError> {
        let working = self
            .working
            .lock()
            .map_err(|_| MemoryError::Serialization("Working memory lock poisoned".to_string()))?;
        Ok(working.active_turns())
    }
}
