use miniai_core::{CloudConsentToken, Modality};
use serde::{Deserialize, Serialize};

/// Lifecycle status of a managed cloud consent token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    /// Token is valid, acknowledged, and within its active time window.
    Active,
    /// Token was explicitly revoked by the user or an emergency kill-switch.
    Revoked,
    /// Token passed its expiration timestamp.
    Expired,
    /// Token has consumed its entire allocated byte payload quota.
    Exhausted,
}

impl std::fmt::Display for ConsentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Revoked => write!(f, "revoked"),
            Self::Expired => write!(f, "expired"),
            Self::Exhausted => write!(f, "exhausted"),
        }
    }
}

/// Actions recorded into the immutable append-only consent audit ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentAction {
    /// User explicitly granted and acknowledged a cloud consent token.
    Granted,
    /// Pre-flight pre-request authorization check succeeded.
    Verified,
    /// Token was explicitly revoked by the user or privacy kill-switch.
    Revoked,
    /// Token reached its expiration timestamp and was transitioned to expired.
    Expired,
    /// An authorization attempt failed (e.g. privacy mode, scope mismatch, quota).
    Rejected,
    /// Remote payload was dispatched under the active token.
    PayloadDispatched,
}

impl std::fmt::Display for ConsentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Granted => write!(f, "granted"),
            Self::Verified => write!(f, "verified"),
            Self::Revoked => write!(f, "revoked"),
            Self::Expired => write!(f, "expired"),
            Self::Rejected => write!(f, "rejected"),
            Self::PayloadDispatched => write!(f, "payload_dispatched"),
        }
    }
}

impl std::str::FromStr for ConsentAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "granted" => Ok(Self::Granted),
            "verified" => Ok(Self::Verified),
            "revoked" => Ok(Self::Revoked),
            "expired" => Ok(Self::Expired),
            "rejected" => Ok(Self::Rejected),
            "payload_dispatched" => Ok(Self::PayloadDispatched),
            other => Err(format!("Unknown ConsentAction: {other}")),
        }
    }
}

/// Scoped permissions bound to a consent token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentScope {
    pub modality: Modality,
    pub target_endpoint: String,
    pub max_payload_bytes: u64,
    pub allowed_operations: Vec<String>,
}

/// High-level wrapper tracking active token lifecycle, cryptographic fingerprint,
/// and cumulative payload consumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedConsentToken {
    /// Core time-bounded token contract.
    pub token: CloudConsentToken,
    /// Current lifecycle status.
    pub status: ConsentStatus,
    /// Cryptographic SHA-256 fingerprint computed across the token definition.
    pub fingerprint: String,
    /// Cumulative payload bytes dispatched using this token.
    pub cumulative_dispatched_bytes: u64,
    /// Optional epoch timestamp when revocation occurred.
    pub revoked_at_epoch_secs: Option<u64>,
    /// Optional human-readable rationale for revocation.
    pub revocation_reason: Option<String>,
}

impl ManagedConsentToken {
    /// Computes whether the token is currently usable at `current_epoch_secs`.
    pub fn is_usable_at(&self, current_epoch_secs: u64) -> bool {
        if self.status != ConsentStatus::Active {
            return false;
        }
        if current_epoch_secs >= self.token.expires_at_epoch_secs {
            return false;
        }
        if self.cumulative_dispatched_bytes >= self.token.max_payload_bytes {
            return false;
        }
        true
    }

    /// Remaining available payload bytes under this token's quota.
    pub fn remaining_bytes(&self) -> u64 {
        self.token
            .max_payload_bytes
            .saturating_sub(self.cumulative_dispatched_bytes)
    }
}

/// An immutable record of an action taken on a consent token in the audit ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentAuditRecord {
    pub id: Option<i64>,
    pub token_id: String,
    pub action: ConsentAction,
    pub modality: Modality,
    pub target_endpoint: String,
    pub payload_bytes: u64,
    pub epoch_secs: u64,
    pub details_json: Option<String>,
}

/// Converts a `Modality` to its canonical snake_case string representation.
pub fn modality_to_wire(modality: Modality) -> &'static str {
    match modality {
        Modality::Llm => "llm",
        Modality::Vision => "vision",
        Modality::ImageGeneration => "image_generation",
        Modality::SpeechAsr => "speech_asr",
        Modality::SpeechTts => "speech_tts",
        Modality::Music => "music",
        Modality::Video => "video",
    }
}

/// Parses a `Modality` from its snake_case string representation.
pub fn modality_from_wire(s: &str) -> Result<Modality, crate::error::CloudError> {
    match s.to_lowercase().as_str() {
        "llm" => Ok(Modality::Llm),
        "vision" => Ok(Modality::Vision),
        "image_generation" | "imagegen" => Ok(Modality::ImageGeneration),
        "speech_asr" | "speechasr" => Ok(Modality::SpeechAsr),
        "speech_tts" | "speechtts" => Ok(Modality::SpeechTts),
        "music" => Ok(Modality::Music),
        "video" => Ok(Modality::Video),
        other => Err(crate::error::CloudError::InvalidToken {
            reason: format!("Unrecognized modality: {other}"),
        }),
    }
}
