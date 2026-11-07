use miniai_core::{AcceleratorBackend, Modality};
use serde::{Deserialize, Serialize};

/// Persisted key-value setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at_epoch_secs: u64,
}

/// Workspace record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at_epoch_secs: u64,
    pub updated_at_epoch_secs: u64,
    pub is_active: bool,
}

/// Immutable record of an action taken on a `CloudConsentToken`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentAuditEvent {
    pub id: Option<i64>,
    pub token_id: String,
    pub action: String,
    pub modality: String,
    pub target_endpoint: String,
    pub payload_bytes: u64,
    pub epoch_secs: u64,
    pub details: Option<String>,
}

/// Recorded system diagnostic or audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEventRecord {
    pub id: Option<i64>,
    pub event_type: String,
    pub severity: String,
    pub source: String,
    pub message: String,
    pub payload_json: Option<String>,
    pub created_at_epoch_secs: u64,
}

/// Cached model metadata record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedModelRecord {
    pub id: String,
    pub modality: Modality,
    pub file_path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub preferred_backend: AcceleratorBackend,
    pub created_at_epoch_secs: u64,
}

/// Model manifest catalog record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelManifestRecord {
    pub model_id: String,
    pub name: String,
    pub modality: Modality,
    pub version: String,
    pub description: String,
    pub source_url: String,
    pub expected_sha256: String,
    pub expected_size_bytes: u64,
    pub format: String,
    pub parameter_count: Option<String>,
    pub quantization: Option<String>,
    pub min_ram_bytes: u64,
    pub recommended_backend: AcceleratorBackend,
    pub created_at_epoch_secs: u64,
}

/// Persisted chat session record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSessionRecord {
    pub id: String,
    pub title: String,
    pub created_at_epoch_secs: u64,
    pub updated_at_epoch_secs: u64,
}

/// Canonical conversation role for chat messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = crate::error::StorageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "system" => Ok(MessageRole::System),
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            other => Err(crate::error::StorageError::SerializationError {
                message: format!("Unknown message role: {other}"),
            }),
        }
    }
}

/// Record of an individual chat message in a conversation session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessageRecord {
    pub id: String,
    pub session_id: String,
    pub parent_id: Option<String>,
    pub role: MessageRole,
    pub content: String,
    pub token_count: usize,
    pub model_id: Option<String>,
    pub created_at_epoch_secs: u64,
    pub updated_at_epoch_secs: u64,
    pub is_pinned: bool,
}

/// Full conversation session export payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSessionExport {
    pub session: ChatSessionRecord,
    pub messages: Vec<ChatMessageRecord>,
    pub exported_at_epoch_secs: u64,
}

/// Database integrity verification report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub is_healthy: bool,
    pub integrity_messages: Vec<String>,
    pub foreign_key_violations: Vec<String>,
}

/// Result of quarantining and recovering a corrupted database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantineResult {
    pub original_path: String,
    pub quarantined_backup_path: String,
    pub recreated_healthy: bool,
}

/// Record of an episodic memory summarizing a historical conversation session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodicMemoryRecord {
    pub id: String,
    pub session_id: String,
    pub topic: String,
    pub summary: String,
    pub key_decisions: String,
    pub created_at_epoch_secs: u64,
}

/// Record of an extracted semantic fact or durable user preference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticFactRecord {
    pub id: String,
    pub category: String,
    pub content: String,
    pub confidence: f64,
    pub created_at_epoch_secs: u64,
    pub updated_at_epoch_secs: u64,
}
