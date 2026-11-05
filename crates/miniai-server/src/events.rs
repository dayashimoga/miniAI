use miniai_core::{AdapterHealth, HardwareProfile, PrivacyPolicyState};
use miniai_storage::DiagnosticEventRecord;
use serde::{Deserialize, Serialize};

/// Strongly typed system events emitted over the MiniAI event bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SystemEvent {
    HardwareChanged {
        profile: HardwareProfile,
    },
    PrivacyPolicyUpdated {
        state: PrivacyPolicyState,
    },
    AdapterHealthChanged {
        health: AdapterHealth,
    },
    DiagnosticAlert {
        record: DiagnosticEventRecord,
    },
    ModelDownloadProgress {
        model_id: String,
        bytes_downloaded: u64,
        total_bytes: u64,
        percentage: f32,
    },
    InferenceTokenStream {
        session_id: String,
        token: String,
        is_final: bool,
    },
    WorkspaceChanged {
        active_workspace_id: String,
    },
}

impl SystemEvent {
    /// Returns the discriminator string of the event.
    pub const fn event_type(&self) -> &'static str {
        match self {
            SystemEvent::HardwareChanged { .. } => "hardware_changed",
            SystemEvent::PrivacyPolicyUpdated { .. } => "privacy_policy_updated",
            SystemEvent::AdapterHealthChanged { .. } => "adapter_health_changed",
            SystemEvent::DiagnosticAlert { .. } => "diagnostic_alert",
            SystemEvent::ModelDownloadProgress { .. } => "model_download_progress",
            SystemEvent::InferenceTokenStream { .. } => "inference_token_stream",
            SystemEvent::WorkspaceChanged { .. } => "workspace_changed",
        }
    }
}

/// Envelope wrapping every published event with a monotonic sequence number and timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub sequence: u64,
    pub timestamp_epoch_secs: u64,
    pub event: SystemEvent,
}
