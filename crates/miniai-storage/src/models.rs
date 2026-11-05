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

/// Persisted chat session record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSessionRecord {
    pub id: String,
    pub title: String,
    pub created_at_epoch_secs: u64,
    pub updated_at_epoch_secs: u64,
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
