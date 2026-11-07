use miniai_storage::{SemanticFactRecord, StorageEngine};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::error::MemoryError;

/// Service managing Tier-3 semantic facts and durable user preferences.
pub struct SemanticMemoryService {
    storage: Arc<StorageEngine>,
}

impl SemanticMemoryService {
    /// Creates a new `SemanticMemoryService` connected to persistent SQLite storage.
    pub fn new(storage: Arc<StorageEngine>) -> Self {
        Self { storage }
    }

    /// Stores a new or updated durable semantic fact.
    pub fn store_fact(
        &self,
        category: &str,
        content: &str,
        confidence: f64,
        epoch_secs: u64,
    ) -> Result<SemanticFactRecord, MemoryError> {
        let trimmed_content = content.trim();
        let trimmed_category = category.trim().to_ascii_lowercase();

        let mut hasher = Sha256::new();
        hasher.update(trimmed_category.as_bytes());
        hasher.update(b":");
        hasher.update(trimmed_content.as_bytes());
        let digest = hasher.finalize();
        let fact_id = format!("fact-{}", &format!("{:x}", digest)[..12]);

        let record = SemanticFactRecord {
            id: fact_id,
            category: trimmed_category,
            content: trimmed_content.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            created_at_epoch_secs: epoch_secs,
            updated_at_epoch_secs: epoch_secs,
        };

        self.storage.insert_semantic_fact(&record)?;
        Ok(record)
    }

    /// Lists all durable semantic facts in memory.
    pub fn list_all(&self) -> Result<Vec<SemanticFactRecord>, MemoryError> {
        self.storage
            .list_semantic_facts()
            .map_err(MemoryError::from)
    }

    /// Queries semantic facts by category and/or content substring search.
    pub fn search(
        &self,
        category: Option<&str>,
        query: Option<&str>,
    ) -> Result<Vec<SemanticFactRecord>, MemoryError> {
        self.storage
            .search_semantic_facts(category, query)
            .map_err(MemoryError::from)
    }

    /// Deletes a specific fact immediately (GDPR "Forget Memory" guarantee).
    pub fn forget(&self, fact_id: &str) -> Result<bool, MemoryError> {
        self.storage
            .forget_semantic_fact(fact_id)
            .map_err(MemoryError::from)
    }
}
