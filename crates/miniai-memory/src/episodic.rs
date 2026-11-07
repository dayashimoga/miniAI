use miniai_storage::{ChatMessageRecord, EpisodicMemoryRecord, StorageEngine};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::error::MemoryError;

/// Service managing Tier-2 episodic memories (cross-session conversation summaries).
pub struct EpisodicMemoryService {
    storage: Arc<StorageEngine>,
}

impl EpisodicMemoryService {
    /// Creates a new `EpisodicMemoryService` connected to persistent SQLite storage.
    pub fn new(storage: Arc<StorageEngine>) -> Self {
        Self { storage }
    }

    /// Automatically summarizes a finished conversation session and records the episodic memory.
    pub fn summarize_and_store_session(
        &self,
        session_id: &str,
        session_title: &str,
        messages: &[ChatMessageRecord],
        created_at_epoch_secs: u64,
    ) -> Result<EpisodicMemoryRecord, MemoryError> {
        let topic = if !session_title.trim().is_empty() {
            session_title.trim().to_string()
        } else {
            format!("Session {session_id}")
        };

        // Extract key decisions and summarize turns
        let mut user_prompts = Vec::new();
        let mut assistant_responses = Vec::new();

        for m in messages {
            match m.role {
                miniai_storage::MessageRole::User => user_prompts.push(m.content.clone()),
                miniai_storage::MessageRole::Assistant => {
                    assistant_responses.push(m.content.clone())
                }
                miniai_storage::MessageRole::System => {}
            }
        }

        let summary = format!(
            "Conversation covered {} user queries and {} assistant responses regarding {}.",
            user_prompts.len(),
            assistant_responses.len(),
            topic
        );

        let key_decisions = if let Some(last_ans) = assistant_responses.last() {
            let truncated = if last_ans.len() > 160 {
                format!("{}...", &last_ans[..157])
            } else {
                last_ans.clone()
            };
            format!("Concluded with: {truncated}")
        } else {
            "No conclusions recorded.".to_string()
        };

        // Generate deterministic episodic record ID
        let mut hasher = Sha256::new();
        hasher.update(session_id.as_bytes());
        hasher.update(created_at_epoch_secs.to_le_bytes());
        let digest = hasher.finalize();
        let record_id = format!("ep-{}", &format!("{:x}", digest)[..12]);

        let record = EpisodicMemoryRecord {
            id: record_id,
            session_id: session_id.to_string(),
            topic,
            summary,
            key_decisions,
            created_at_epoch_secs,
        };

        self.storage.insert_episodic_memory(&record)?;
        Ok(record)
    }

    /// Lists recent episodic memories ordered by timestamp descending.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<EpisodicMemoryRecord>, MemoryError> {
        self.storage
            .list_episodic_memories(limit)
            .map_err(MemoryError::from)
    }

    /// Searches episodic memories matching query keywords.
    pub fn search(&self, query: &str) -> Result<Vec<EpisodicMemoryRecord>, MemoryError> {
        self.storage
            .search_episodic_memories(query)
            .map_err(MemoryError::from)
    }
}
