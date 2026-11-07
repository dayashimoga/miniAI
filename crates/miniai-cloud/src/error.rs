use thiserror::Error;

/// Domain error type for cloud consent governance and remote compute dispatch.
#[derive(Debug, Error)]
pub enum CloudError {
    #[error("Outbound cloud execution blocked: Local-Only Privacy Mode is active")]
    PrivacyModeBlocked,

    #[error("Cloud consent token required: no active user-acknowledged token found")]
    ConsentRequired,

    #[error(
        "Cloud consent token has expired (current_epoch={current_epoch}, expires_at={expires_at})"
    )]
    ConsentExpired { current_epoch: u64, expires_at: u64 },

    #[error("Cloud consent token '{token_id}' has been revoked: {reason}")]
    ConsentRevoked { token_id: String, reason: String },

    #[error("Cloud consent scope mismatch: expected {expected}, actual {actual}")]
    ScopeMismatch { expected: String, actual: String },

    #[error(
        "Request payload ({requested_bytes} bytes) exceeds token limit ({allowed_bytes} bytes)"
    )]
    PayloadLimitExceeded {
        allowed_bytes: u64,
        requested_bytes: u64,
    },

    #[error("Cumulative token payload quota exceeded (used {total_used} + requested {requested} > max {max_allowed} bytes)")]
    QuotaExceeded {
        total_used: u64,
        requested: u64,
        max_allowed: u64,
    },

    #[error("Insecure endpoint '{endpoint}': target endpoint must use https://")]
    InsecureEndpoint { endpoint: String },

    #[error("Invalid cloud consent token: {reason}")]
    InvalidToken { reason: String },

    #[error("Consent token not found: '{token_id}'")]
    TokenNotFound { token_id: String },

    #[error("Remote transport error: {reason}")]
    RemoteTransportError { reason: String },

    #[error("Remote worker error (status {status_code}): {message}")]
    RemoteWorkerError { status_code: u16, message: String },

    #[error("TLS/mTLS configuration error: {reason}")]
    TlsConfigurationError { reason: String },

    #[error("Local fallback execution failed: {reason}")]
    FallbackFailed { reason: String },

    #[error("Compute operation was cancelled")]
    OperationCancelled,

    #[error("Storage error: {0}")]
    Storage(#[from] miniai_storage::StorageError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Core error: {0}")]
    Core(#[from] miniai_core::MiniAiError),
}
