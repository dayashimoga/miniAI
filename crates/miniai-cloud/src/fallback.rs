use crate::dispatcher::{ComputeTaskRequest, ComputeTaskResponse, RemoteDispatcher};
use crate::error::CloudError;
use crate::ledger::ConsentLedger;
use crate::types::{ConsentAction, ConsentAuditRecord};
use miniai_core::PrivacyPolicyEngine;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

/// Trait defining local execution fallback logic for all modalities on CPU.
pub trait LocalFallbackHandler: Send + Sync {
    /// Executes the compute task on the local CPU runtime adapter.
    fn execute_local(&self, req: &ComputeTaskRequest) -> Result<Vec<u8>, CloudError>;
}

/// Default local CPU fallback executor producing deterministic local outputs.
#[derive(Default)]
pub struct DefaultLocalFallbackHandler {
    pub custom_output: Option<Vec<u8>>,
}

impl LocalFallbackHandler for DefaultLocalFallbackHandler {
    fn execute_local(&self, req: &ComputeTaskRequest) -> Result<Vec<u8>, CloudError> {
        if let Some(out) = &self.custom_output {
            return Ok(out.clone());
        }

        let mut out = req.payload.clone();
        out.extend_from_slice(b":processed_locally_cpu");
        Ok(out)
    }
}

/// Coordinates hybrid remote compute dispatch with instant, automatic local CPU fallback.
pub struct FallbackCoordinator {
    dispatcher: Arc<RemoteDispatcher>,
    local_handler: Arc<dyn LocalFallbackHandler>,
    ledger: Arc<ConsentLedger>,
}

impl FallbackCoordinator {
    /// Creates a new `FallbackCoordinator`.
    pub fn new(
        dispatcher: Arc<RemoteDispatcher>,
        local_handler: Arc<dyn LocalFallbackHandler>,
        ledger: Arc<ConsentLedger>,
    ) -> Self {
        Self {
            dispatcher,
            local_handler,
            ledger,
        }
    }

    /// Attempts remote execution on cloud GPU; if privacy mode blocks it, consent is revoked,
    /// or remote network/worker errors occur, automatically falls back to local CPU execution.
    pub fn execute(
        &self,
        endpoint: &str,
        req: ComputeTaskRequest,
        current_epoch_secs: u64,
        privacy: &PrivacyPolicyEngine,
        cancellation_token: Option<&AtomicBool>,
    ) -> Result<ComputeTaskResponse, CloudError> {
        let start = Instant::now();

        // Immediate cancellation check
        if cancellation_token.is_some_and(|c| c.load(std::sync::atomic::Ordering::SeqCst)) {
            return Err(CloudError::OperationCancelled);
        }

        // Attempt remote dispatch
        match self.dispatcher.dispatch_task(
            endpoint,
            req.clone(),
            current_epoch_secs,
            privacy,
            cancellation_token,
        ) {
            Ok(remote_resp) => Ok(remote_resp),
            Err(err) => {
                // If the operation was explicitly cancelled by the user, fail immediately without fallback
                if matches!(err, CloudError::OperationCancelled) {
                    return Err(err);
                }

                let reason = err.to_string();

                // Execute on local CPU fallback handler
                let local_output = self.local_handler.execute_local(&req).map_err(|e| {
                    CloudError::FallbackFailed {
                        reason: format!("Local CPU fallback handler failed: {e}"),
                    }
                })?;

                let latency_ms = start.elapsed().as_millis() as u64;

                // Record fallback audit event in the ledger
                let audit = ConsentAuditRecord {
                    id: None,
                    token_id: req.token_id.clone(),
                    action: ConsentAction::Rejected,
                    modality: req.modality,
                    target_endpoint: endpoint.to_string(),
                    payload_bytes: req.payload.len() as u64,
                    epoch_secs: current_epoch_secs,
                    details_json: Some(format!(
                        r#"{{"fallback":"local_cpu","reason":"{reason}","latency_ms":{latency_ms}}}"#
                    )),
                };
                let _ = self.ledger.record_event(&audit);

                Ok(ComputeTaskResponse {
                    task_id: req.task_id,
                    worker_id: "local_cpu_adapter".to_string(),
                    status: "success_fallback".to_string(),
                    output_bytes: local_output,
                    latency_ms,
                    executed_remotely: false,
                    fallback_reason: Some(reason),
                })
            }
        }
    }
}
