use crate::error::CloudError;
use crate::ledger::ConsentLedger;
use crate::token::TokenValidator;
use crate::types::{ConsentAction, ConsentAuditRecord, ConsentStatus, ManagedConsentToken};
use miniai_core::{CloudConsentToken, Modality, PrivacyPolicyEngine};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe coordinator managing the scoped lifecycle, verification, revocation,
/// payload consumption, and audit logging of cloud consent tokens.
pub struct ConsentManager {
    ledger: Arc<ConsentLedger>,
    tokens: RwLock<HashMap<String, ManagedConsentToken>>,
    allow_loopback: bool,
}

impl ConsentManager {
    /// Creates a new `ConsentManager` with strict production validation (no loopback endpoints).
    pub fn new(ledger: Arc<ConsentLedger>) -> Self {
        Self {
            ledger,
            tokens: RwLock::new(HashMap::new()),
            allow_loopback: false,
        }
    }

    /// Creates a new `ConsentManager` with configurable loopback permission for local tests.
    pub fn new_with_loopback(ledger: Arc<ConsentLedger>, allow_loopback: bool) -> Self {
        Self {
            ledger,
            tokens: RwLock::new(HashMap::new()),
            allow_loopback,
        }
    }

    /// Returns a reference to the underlying audit ledger.
    pub fn ledger(&self) -> &ConsentLedger {
        &self.ledger
    }

    /// Validates, registers, and audits the explicit granting of a `CloudConsentToken`.
    pub fn grant_consent(
        &self,
        token: CloudConsentToken,
        current_epoch_secs: u64,
        privacy: &mut PrivacyPolicyEngine,
    ) -> Result<ManagedConsentToken, CloudError> {
        // Enforce Local-Only Privacy Mode check first
        if privacy.is_local_only() {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token.token_id.clone(),
                action: ConsentAction::Rejected,
                modality: token.modality,
                target_endpoint: token.target_endpoint.clone(),
                payload_bytes: 0,
                epoch_secs: current_epoch_secs,
                details_json: Some(
                    r#"{"reason":"LocalOnlyPrivacyMode is active; grant rejected"}"#.to_string(),
                ),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::PrivacyModeBlocked);
        }

        // Validate token structural, security, and payload boundaries
        TokenValidator::validate(&token, self.allow_loopback)?;

        let fingerprint = TokenValidator::compute_token_fingerprint(&token);

        let managed = ManagedConsentToken {
            token: token.clone(),
            status: ConsentStatus::Active,
            fingerprint: fingerprint.clone(),
            cumulative_dispatched_bytes: 0,
            revoked_at_epoch_secs: None,
            revocation_reason: None,
        };

        // Sync with core PrivacyPolicyEngine
        privacy.grant_cloud_consent(token.clone())?;

        // Persist in-memory managed registry
        {
            let mut lock = self.tokens.write().unwrap();
            lock.insert(token.token_id.clone(), managed.clone());
        }

        // Record immutable audit event
        let audit = ConsentAuditRecord {
            id: None,
            token_id: token.token_id.clone(),
            action: ConsentAction::Granted,
            modality: token.modality,
            target_endpoint: token.target_endpoint.clone(),
            payload_bytes: 0,
            epoch_secs: current_epoch_secs,
            details_json: Some(format!(
                r#"{{"fingerprint":"{}","max_payload_bytes":{}}}"#,
                fingerprint, token.max_payload_bytes
            )),
        };
        self.ledger.record_event(&audit)?;

        Ok(managed)
    }

    /// Verifies whether an outbound remote execution request is permitted under the scoped token.
    pub fn verify_and_authorize(
        &self,
        token_id: &str,
        modality: Modality,
        endpoint: &str,
        payload_bytes: u64,
        current_epoch_secs: u64,
        privacy: &PrivacyPolicyEngine,
    ) -> Result<(), CloudError> {
        // Check core privacy mode
        if privacy.is_local_only() {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(
                    r#"{"reason":"Local-Only Privacy Mode is active; execution blocked"}"#
                        .to_string(),
                ),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::PrivacyModeBlocked);
        }

        let mut lock = self.tokens.write().unwrap();
        let managed = match lock.get_mut(token_id) {
            Some(t) => t,
            None => {
                let record = ConsentAuditRecord {
                    id: None,
                    token_id: token_id.to_string(),
                    action: ConsentAction::Rejected,
                    modality,
                    target_endpoint: endpoint.to_string(),
                    payload_bytes,
                    epoch_secs: current_epoch_secs,
                    details_json: Some(
                        r#"{"reason":"Token ID not found in managed registry"}"#.to_string(),
                    ),
                };
                let _ = self.ledger.record_event(&record);
                return Err(CloudError::TokenNotFound {
                    token_id: token_id.to_string(),
                });
            }
        };

        // Check revocation
        if managed.status == ConsentStatus::Revoked {
            let reason = managed
                .revocation_reason
                .clone()
                .unwrap_or_else(|| "User revoked".to_string());
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(format!(r#"{{"reason":"Token was revoked: {reason}"}}"#)),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::ConsentRevoked {
                token_id: token_id.to_string(),
                reason,
            });
        }

        // Check expiration
        if managed.status == ConsentStatus::Expired
            || current_epoch_secs >= managed.token.expires_at_epoch_secs
        {
            managed.status = ConsentStatus::Expired;
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Expired,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(format!(
                    r#"{{"current_epoch":{},"expires_at":{}}}"#,
                    current_epoch_secs, managed.token.expires_at_epoch_secs
                )),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::ConsentExpired {
                current_epoch: current_epoch_secs,
                expires_at: managed.token.expires_at_epoch_secs,
            });
        }

        // Check exhaustion
        if managed.status == ConsentStatus::Exhausted {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(
                    r#"{"reason":"Token byte quota has been completely exhausted"}"#.to_string(),
                ),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::QuotaExceeded {
                total_used: managed.cumulative_dispatched_bytes,
                requested: payload_bytes,
                max_allowed: managed.token.max_payload_bytes,
            });
        }

        // Check modality match
        if managed.token.modality != modality {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(format!(
                    r#"{{"expected_modality":"{}","actual_modality":"{}"}}"#,
                    crate::types::modality_to_wire(managed.token.modality),
                    crate::types::modality_to_wire(modality)
                )),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::ScopeMismatch {
                expected: crate::types::modality_to_wire(managed.token.modality).to_string(),
                actual: crate::types::modality_to_wire(modality).to_string(),
            });
        }

        // Check endpoint match (prefix or exact hostname)
        let token_base = managed
            .token
            .target_endpoint
            .trim_end_matches('/')
            .to_lowercase();
        let target_base = endpoint.trim_end_matches('/').to_lowercase();
        if !target_base.starts_with(&token_base) {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(format!(
                    r#"{{"expected_endpoint":"{}","actual_endpoint":"{}"}}"#,
                    managed.token.target_endpoint, endpoint
                )),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::ScopeMismatch {
                expected: managed.token.target_endpoint.clone(),
                actual: endpoint.to_string(),
            });
        }

        // Check payload bytes ceiling
        if payload_bytes > managed.token.max_payload_bytes {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(format!(
                    r#"{{"requested_bytes":{},"max_allowed":{}}}"#,
                    payload_bytes, managed.token.max_payload_bytes
                )),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::PayloadLimitExceeded {
                allowed_bytes: managed.token.max_payload_bytes,
                requested_bytes: payload_bytes,
            });
        }

        // Check cumulative quota
        if managed
            .cumulative_dispatched_bytes
            .saturating_add(payload_bytes)
            > managed.token.max_payload_bytes
        {
            let record = ConsentAuditRecord {
                id: None,
                token_id: token_id.to_string(),
                action: ConsentAction::Rejected,
                modality,
                target_endpoint: endpoint.to_string(),
                payload_bytes,
                epoch_secs: current_epoch_secs,
                details_json: Some(format!(
                    r#"{{"used":{},"requested":{},"max":{}}}"#,
                    managed.cumulative_dispatched_bytes,
                    payload_bytes,
                    managed.token.max_payload_bytes
                )),
            };
            let _ = self.ledger.record_event(&record);
            return Err(CloudError::QuotaExceeded {
                total_used: managed.cumulative_dispatched_bytes,
                requested: payload_bytes,
                max_allowed: managed.token.max_payload_bytes,
            });
        }

        // Record verification success in ledger
        let record = ConsentAuditRecord {
            id: None,
            token_id: token_id.to_string(),
            action: ConsentAction::Verified,
            modality,
            target_endpoint: endpoint.to_string(),
            payload_bytes,
            epoch_secs: current_epoch_secs,
            details_json: Some(format!(
                r#"{{"status":"preflight_verified","remaining_bytes":{}}}"#,
                managed.remaining_bytes()
            )),
        };
        self.ledger.record_event(&record)?;

        Ok(())
    }

    /// Records the successful dispatch of a payload, consuming bytes from the token quota.
    pub fn record_dispatch(
        &self,
        token_id: &str,
        payload_bytes: u64,
        current_epoch_secs: u64,
    ) -> Result<(), CloudError> {
        let mut lock = self.tokens.write().unwrap();
        let managed = lock
            .get_mut(token_id)
            .ok_or_else(|| CloudError::TokenNotFound {
                token_id: token_id.to_string(),
            })?;

        managed.cumulative_dispatched_bytes = managed
            .cumulative_dispatched_bytes
            .saturating_add(payload_bytes);

        if managed.cumulative_dispatched_bytes >= managed.token.max_payload_bytes {
            managed.status = ConsentStatus::Exhausted;
        }

        let record = ConsentAuditRecord {
            id: None,
            token_id: token_id.to_string(),
            action: ConsentAction::PayloadDispatched,
            modality: managed.token.modality,
            target_endpoint: managed.token.target_endpoint.clone(),
            payload_bytes,
            epoch_secs: current_epoch_secs,
            details_json: Some(format!(
                r#"{{"cumulative_bytes":{},"status":"{}"}}"#,
                managed.cumulative_dispatched_bytes, managed.status
            )),
        };
        self.ledger.record_event(&record)?;

        Ok(())
    }

    /// Explicitly revokes an active token.
    pub fn revoke_consent(
        &self,
        token_id: &str,
        reason: &str,
        current_epoch_secs: u64,
        privacy: &mut PrivacyPolicyEngine,
    ) -> Result<(), CloudError> {
        let mut lock = self.tokens.write().unwrap();
        let managed = lock
            .get_mut(token_id)
            .ok_or_else(|| CloudError::TokenNotFound {
                token_id: token_id.to_string(),
            })?;

        managed.status = ConsentStatus::Revoked;
        managed.revoked_at_epoch_secs = Some(current_epoch_secs);
        managed.revocation_reason = Some(reason.to_string());

        // Revoke in core PrivacyPolicyEngine if active
        if let Some(active) = &privacy.state().active_consent {
            if active.token_id == token_id {
                privacy.revoke_cloud_consent();
            }
        }

        let record = ConsentAuditRecord {
            id: None,
            token_id: token_id.to_string(),
            action: ConsentAction::Revoked,
            modality: managed.token.modality,
            target_endpoint: managed.token.target_endpoint.clone(),
            payload_bytes: 0,
            epoch_secs: current_epoch_secs,
            details_json: Some(format!(r#"{{"reason":"{reason}"}}"#)),
        };
        self.ledger.record_event(&record)?;

        Ok(())
    }

    /// Emergency kill-switch revoking all currently active tokens and disabling outbound permissions.
    pub fn revoke_all(
        &self,
        reason: &str,
        current_epoch_secs: u64,
        privacy: &mut PrivacyPolicyEngine,
    ) -> Result<usize, CloudError> {
        privacy.revoke_cloud_consent();

        let mut lock = self.tokens.write().unwrap();
        let mut count = 0;

        for (tid, managed) in lock.iter_mut() {
            if managed.status == ConsentStatus::Active {
                managed.status = ConsentStatus::Revoked;
                managed.revoked_at_epoch_secs = Some(current_epoch_secs);
                managed.revocation_reason = Some(reason.to_string());

                let record = ConsentAuditRecord {
                    id: None,
                    token_id: tid.clone(),
                    action: ConsentAction::Revoked,
                    modality: managed.token.modality,
                    target_endpoint: managed.token.target_endpoint.clone(),
                    payload_bytes: 0,
                    epoch_secs: current_epoch_secs,
                    details_json: Some(format!(r#"{{"kill_switch":true,"reason":"{reason}"}}"#)),
                };
                let _ = self.ledger.record_event(&record);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Returns a snapshot of a token if it exists.
    pub fn get_token(&self, token_id: &str) -> Option<ManagedConsentToken> {
        let lock = self.tokens.read().unwrap();
        lock.get(token_id).cloned()
    }

    /// Returns all tokens that are currently active and unexpired at `current_epoch_secs`.
    pub fn list_active_tokens(&self, current_epoch_secs: u64) -> Vec<ManagedConsentToken> {
        let lock = self.tokens.read().unwrap();
        lock.values()
            .filter(|t| t.is_usable_at(current_epoch_secs))
            .cloned()
            .collect()
    }

    /// Retrieves all audit ledger records associated with a specific token.
    pub fn audit_history(&self, token_id: &str) -> Result<Vec<ConsentAuditRecord>, CloudError> {
        self.ledger.query_by_token(token_id)
    }
}
