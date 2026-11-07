use crate::error::CloudError;
use crate::types::{ConsentAction, ConsentAuditRecord};
use miniai_core::Modality;
use miniai_storage::{ConsentAuditEvent, StorageEngine};
use std::str::FromStr;
use std::sync::Arc;

/// Immutable append-only audit ledger for all cloud consent token actions.
pub struct ConsentLedger {
    storage: Arc<StorageEngine>,
}

impl ConsentLedger {
    /// Creates a new `ConsentLedger` wrapping a shared `StorageEngine`.
    pub fn new(storage: Arc<StorageEngine>) -> Self {
        Self { storage }
    }

    /// Records an immutable consent audit event into the SQLite audit ledger.
    pub fn record_event(&self, record: &ConsentAuditRecord) -> Result<i64, CloudError> {
        let storage_event = ConsentAuditEvent {
            id: None,
            token_id: record.token_id.clone(),
            action: record.action.to_string(),
            modality: crate::types::modality_to_wire(record.modality).to_string(),
            target_endpoint: record.target_endpoint.clone(),
            payload_bytes: record.payload_bytes,
            epoch_secs: record.epoch_secs,
            details: record.details_json.clone(),
        };

        let row_id = self.storage.record_consent_event(&storage_event)?;
        Ok(row_id)
    }

    /// Lists the most recent consent audit records up to `limit`.
    pub fn list_events(&self, limit: usize) -> Result<Vec<ConsentAuditRecord>, CloudError> {
        let raw = self.storage.list_consent_events(limit)?;
        let mut records = Vec::with_capacity(raw.len());
        for ev in raw {
            records.push(Self::convert_storage_event(ev)?);
        }
        Ok(records)
    }

    /// Returns the total count of audit events in the ledger.
    pub fn count_events(&self) -> Result<usize, CloudError> {
        let count = self.storage.count_consent_events()?;
        Ok(count)
    }

    /// Queries audit events filtered by `token_id`.
    pub fn query_by_token(&self, token_id: &str) -> Result<Vec<ConsentAuditRecord>, CloudError> {
        let all = self.list_events(1000)?;
        Ok(all.into_iter().filter(|r| r.token_id == token_id).collect())
    }

    /// Queries audit events filtered by `action`.
    pub fn query_by_action(
        &self,
        action: ConsentAction,
    ) -> Result<Vec<ConsentAuditRecord>, CloudError> {
        let all = self.list_events(1000)?;
        Ok(all.into_iter().filter(|r| r.action == action).collect())
    }

    /// Queries audit events filtered by `modality`.
    pub fn query_by_modality(
        &self,
        modality: Modality,
    ) -> Result<Vec<ConsentAuditRecord>, CloudError> {
        let all = self.list_events(1000)?;
        Ok(all.into_iter().filter(|r| r.modality == modality).collect())
    }

    /// Queries audit events within a given epoch timestamp interval `[start_epoch, end_epoch]`.
    pub fn query_by_time_range(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<Vec<ConsentAuditRecord>, CloudError> {
        let all = self.list_events(1000)?;
        Ok(all
            .into_iter()
            .filter(|r| r.epoch_secs >= start_epoch && r.epoch_secs <= end_epoch)
            .collect())
    }

    fn convert_storage_event(event: ConsentAuditEvent) -> Result<ConsentAuditRecord, CloudError> {
        let action = ConsentAction::from_str(&event.action)
            .map_err(|e| CloudError::InvalidToken { reason: e })?;

        let modality = crate::types::modality_from_wire(&event.modality)?;

        Ok(ConsentAuditRecord {
            id: event.id,
            token_id: event.token_id,
            action,
            modality,
            target_endpoint: event.target_endpoint,
            payload_bytes: event.payload_bytes,
            epoch_secs: event.epoch_secs,
            details_json: event.details,
        })
    }
}
