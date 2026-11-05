use std::sync::Arc;
use tokio::sync::RwLock;

use miniai_core::{
    CloudConsentToken, HardwareDetector, HardwareProfile, PrivacyPolicyEngine, PrivacyPolicyState,
    RuntimeRegistry, SystemSnapshot, VerificationTier,
};
use miniai_storage::{AppSetting, ConsentAuditEvent, StorageEngine};
use serde::{Deserialize, Serialize};

use crate::bus::EventBus;
use crate::error::ServerError;
use crate::events::{EventEnvelope, SystemEvent};

/// Payload for mutating privacy and execution policy settings via IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PrivacyPolicyUpdatePayload {
    pub local_only_mode: Option<bool>,
    pub force_cpu_only: Option<bool>,
    pub grant_consent: Option<CloudConsentToken>,
    pub revoke_consent: Option<bool>,
}

/// In-process IPC bridge coordinating core services, storage, and the event bus.
#[derive(Clone)]
pub struct IpcBridge {
    storage: StorageEngine,
    event_bus: EventBus,
    privacy_engine: Arc<RwLock<PrivacyPolicyEngine>>,
    runtime_registry: Arc<RwLock<RuntimeRegistry>>,
    hardware: HardwareProfile,
    session_nonce: String,
}

impl IpcBridge {
    /// Creates a new `IpcBridge` initialized with real core services, default CPU adapters, and storage.
    pub fn new(
        storage: StorageEngine,
        event_bus: EventBus,
        session_nonce: String,
    ) -> Result<Self, ServerError> {
        let registry = RuntimeRegistry::with_default_cpu_adapters().map_err(|e| {
            ServerError::InternalError {
                message: format!("Failed to initialize default CPU modality adapters: {e}"),
            }
        })?;

        let mut privacy = PrivacyPolicyEngine::new();

        // Restore settings from storage if previously saved
        if let Ok(Some(setting)) = storage.get_setting("local_only_mode") {
            privacy.set_local_only_mode(setting.value == "true");
        }
        if let Ok(Some(setting)) = storage.get_setting("force_cpu_only") {
            privacy.set_force_cpu_only(setting.value == "true");
        }

        let hardware = HardwareDetector::detect_host_profile();

        Ok(Self {
            storage,
            event_bus,
            privacy_engine: Arc::new(RwLock::new(privacy)),
            runtime_registry: Arc::new(RwLock::new(registry)),
            hardware,
            session_nonce,
        })
    }

    /// Returns the session bearer nonce required for authentication.
    pub fn session_nonce(&self) -> &str {
        &self.session_nonce
    }

    /// Verifies candidate bearer token against session nonce.
    pub fn verify_nonce(&self, candidate: &str) -> Result<(), ServerError> {
        let cleaned = candidate
            .trim()
            .strip_prefix("Bearer ")
            .unwrap_or(candidate)
            .trim();
        if cleaned == self.session_nonce && !cleaned.is_empty() {
            Ok(())
        } else {
            Err(ServerError::Unauthorized)
        }
    }

    /// Returns a lightweight health and version status payload.
    pub fn get_health(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "ok",
            "version": "0.1.0",
            "service": "miniai-server",
            "active_subscribers": self.event_bus.subscriber_count(),
            "last_event_sequence": self.event_bus.last_sequence(),
        })
    }

    /// Builds a full `SystemSnapshot` reflecting live hardware, privacy, and modality adapters.
    pub async fn get_system_snapshot(&self) -> Result<SystemSnapshot, ServerError> {
        let privacy = self.privacy_engine.read().await.state().clone();
        let registry = self.runtime_registry.read().await;

        registry
            .build_snapshot(
                self.hardware.clone(),
                privacy,
                VerificationTier::ProvenOnHost,
            )
            .map_err(|e| ServerError::InternalError {
                message: format!("Failed to build system snapshot: {e}"),
            })
    }

    /// Returns the current privacy policy state.
    pub async fn get_privacy_policy(&self) -> PrivacyPolicyState {
        self.privacy_engine.read().await.state().clone()
    }

    /// Updates privacy policy settings, persists to storage, and emits a `PrivacyPolicyUpdated` event.
    pub async fn update_privacy_policy(
        &self,
        payload: PrivacyPolicyUpdatePayload,
        current_epoch_secs: u64,
    ) -> Result<PrivacyPolicyState, ServerError> {
        let mut engine = self.privacy_engine.write().await;

        if let Some(local_only) = payload.local_only_mode {
            engine.set_local_only_mode(local_only);
            let _ = self.storage.set_setting(&AppSetting {
                key: "local_only_mode".to_string(),
                value: if local_only { "true" } else { "false" }.to_string(),
                updated_at_epoch_secs: current_epoch_secs,
            });
        }

        if let Some(force_cpu) = payload.force_cpu_only {
            engine.set_force_cpu_only(force_cpu);
            let _ = self.storage.set_setting(&AppSetting {
                key: "force_cpu_only".to_string(),
                value: if force_cpu { "true" } else { "false" }.to_string(),
                updated_at_epoch_secs: current_epoch_secs,
            });
        }

        if let Some(token) = payload.grant_consent {
            let token_id = token.token_id.clone();
            let modality = format!("{:?}", token.modality);
            let endpoint = token.target_endpoint.clone();
            let bytes = token.max_payload_bytes;

            engine
                .grant_cloud_consent(token)
                .map_err(|e| ServerError::BadRequest {
                    message: format!("Failed to grant cloud consent: {e}"),
                })?;

            let _ = self.storage.record_consent_event(&ConsentAuditEvent {
                id: None,
                token_id,
                action: "GRANT".to_string(),
                modality,
                target_endpoint: endpoint,
                payload_bytes: bytes,
                epoch_secs: current_epoch_secs,
                details: Some("Cloud consent granted via IPC API".to_string()),
            });
        }

        if payload.revoke_consent == Some(true) {
            if let Some(active) = engine.state().active_consent.as_ref() {
                let token_id = active.token_id.clone();
                let modality = format!("{:?}", active.modality);
                let endpoint = active.target_endpoint.clone();
                let _ = self.storage.record_consent_event(&ConsentAuditEvent {
                    id: None,
                    token_id,
                    action: "REVOKE".to_string(),
                    modality,
                    target_endpoint: endpoint,
                    payload_bytes: 0,
                    epoch_secs: current_epoch_secs,
                    details: Some("Cloud consent explicitly revoked via IPC API".to_string()),
                });
            }
            engine.revoke_cloud_consent();
        }

        let updated_state = engine.state().clone();

        // Publish event to broadcast bus
        let _ = self
            .event_bus
            .publish(
                SystemEvent::PrivacyPolicyUpdated {
                    state: updated_state.clone(),
                },
                current_epoch_secs,
            )
            .await;

        Ok(updated_state)
    }

    /// Returns a list of recently buffered system events.
    pub async fn get_recent_events(&self) -> Vec<EventEnvelope> {
        self.event_bus.recent_events().await
    }

    /// Returns a reference to the event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Returns a reference to the storage engine.
    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }
}
