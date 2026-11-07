use crate::error::CloudError;
use crate::manager::ConsentManager;
use miniai_core::{Modality, PrivacyPolicyEngine};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Mutual TLS (mTLS) and TLS 1.3 configuration required for secure cloud worker dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Client certificate in PEM format for mutual authentication.
    pub client_cert_pem: Option<String>,
    /// Client private key in PEM format.
    pub client_key_pem: Option<String>,
    /// Root Certificate Authority (CA) bundle in PEM format.
    pub ca_cert_pem: Option<String>,
    /// Whether mutual TLS (client cert presentation) is mandatory.
    pub require_mtls: bool,
    /// Minimum TLS protocol version (must be "TLSv1.3").
    pub tls_min_version: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            client_cert_pem: None,
            client_key_pem: None,
            ca_cert_pem: None,
            require_mtls: false,
            tls_min_version: "TLSv1.3".to_string(),
        }
    }
}

impl TlsConfig {
    /// Validates the TLS configuration bounds and cryptographic PEM headers.
    pub fn validate(&self) -> Result<(), CloudError> {
        if self.tls_min_version != "TLSv1.3" {
            return Err(CloudError::TlsConfigurationError {
                reason: format!(
                    "Minimum TLS version must be 'TLSv1.3', found '{}'",
                    self.tls_min_version
                ),
            });
        }

        if self.require_mtls {
            let cert = self.client_cert_pem.as_deref().unwrap_or("").trim();
            if !cert.contains("-----BEGIN CERTIFICATE-----")
                || !cert.contains("-----END CERTIFICATE-----")
            {
                return Err(CloudError::TlsConfigurationError {
                    reason: "mTLS is required but client_cert_pem is missing valid PEM certificate markers"
                        .to_string(),
                });
            }

            let key = self.client_key_pem.as_deref().unwrap_or("").trim();
            if !key.contains("PRIVATE KEY-----") {
                return Err(CloudError::TlsConfigurationError {
                    reason: "mTLS is required but client_key_pem is missing valid PEM private key markers"
                        .to_string(),
                });
            }
        }

        if let Some(ca) = &self.ca_cert_pem {
            let ca_str = ca.trim();
            if !ca_str.is_empty()
                && (!ca_str.contains("-----BEGIN CERTIFICATE-----")
                    || !ca_str.contains("-----END CERTIFICATE-----"))
            {
                return Err(CloudError::TlsConfigurationError {
                    reason: "ca_cert_pem is provided but missing valid PEM certificate markers"
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Request envelope submitted to a remote GPU compute worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeTaskRequest {
    pub task_id: String,
    pub token_id: String,
    pub modality: Modality,
    pub operation: String,
    pub payload: Vec<u8>,
    pub timeout_ms: u64,
}

/// Result returned from remote compute execution or local fallback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeTaskResponse {
    pub task_id: String,
    pub worker_id: String,
    pub status: String,
    pub output_bytes: Vec<u8>,
    pub latency_ms: u64,
    pub executed_remotely: bool,
    pub fallback_reason: Option<String>,
}

/// Trait abstraction for remote worker network communication.
pub trait RemoteWorkerTransport: Send + Sync {
    /// Dispatches the request envelope to the remote endpoint.
    fn dispatch(
        &self,
        endpoint: &str,
        req: &ComputeTaskRequest,
    ) -> Result<ComputeTaskResponse, CloudError>;
}

/// Simulated remote transport for testing network latencies, mTLS, remote errors, and timeouts.
pub struct SimulatedRemoteTransport {
    pub worker_id: String,
    pub simulated_latency_ms: u64,
    pub should_fail_with_500: bool,
    pub should_timeout: bool,
    pub should_drop_connection: bool,
    pub custom_output: Option<Vec<u8>>,
}

impl Default for SimulatedRemoteTransport {
    fn default() -> Self {
        Self {
            worker_id: "remote-worker-node-01".to_string(),
            simulated_latency_ms: 10,
            should_fail_with_500: false,
            should_timeout: false,
            should_drop_connection: false,
            custom_output: None,
        }
    }
}

impl RemoteWorkerTransport for SimulatedRemoteTransport {
    fn dispatch(
        &self,
        endpoint: &str,
        req: &ComputeTaskRequest,
    ) -> Result<ComputeTaskResponse, CloudError> {
        let ep = endpoint.trim();
        if !ep.starts_with("https://") {
            return Err(CloudError::InsecureEndpoint {
                endpoint: endpoint.to_string(),
            });
        }

        if self.should_drop_connection {
            return Err(CloudError::RemoteTransportError {
                reason: "Connection reset by remote peer".to_string(),
            });
        }

        if self.should_timeout {
            return Err(CloudError::RemoteTransportError {
                reason: format!(
                    "Remote worker response timed out after {}ms",
                    req.timeout_ms
                ),
            });
        }

        if self.should_fail_with_500 {
            return Err(CloudError::RemoteWorkerError {
                status_code: 500,
                message: "Internal remote accelerator out-of-memory error".to_string(),
            });
        }

        let output = self.custom_output.clone().unwrap_or_else(|| {
            // Default echo / transform for testing
            let mut out = req.payload.clone();
            out.extend_from_slice(b":processed_remotely");
            out
        });

        Ok(ComputeTaskResponse {
            task_id: req.task_id.clone(),
            worker_id: self.worker_id.clone(),
            status: "success".to_string(),
            output_bytes: output,
            latency_ms: self.simulated_latency_ms,
            executed_remotely: true,
            fallback_reason: None,
        })
    }
}

/// Dispatches compute workloads to remote GPU workers gated by pre-flight consent.
pub struct RemoteDispatcher {
    consent_manager: Arc<ConsentManager>,
    tls_config: TlsConfig,
    transport: Arc<dyn RemoteWorkerTransport>,
}

impl RemoteDispatcher {
    /// Creates a new `RemoteDispatcher`.
    pub fn new(
        consent_manager: Arc<ConsentManager>,
        tls_config: TlsConfig,
        transport: Arc<dyn RemoteWorkerTransport>,
    ) -> Self {
        Self {
            consent_manager,
            tls_config,
            transport,
        }
    }

    /// Returns a reference to the TLS configuration.
    pub fn tls_config(&self) -> &TlsConfig {
        &self.tls_config
    }

    /// Dispatches a compute task to a remote worker, strictly enforcing pre-flight consent
    /// and mid-flight cancellation guarantees.
    pub fn dispatch_task(
        &self,
        endpoint: &str,
        req: ComputeTaskRequest,
        current_epoch_secs: u64,
        privacy: &PrivacyPolicyEngine,
        cancellation_token: Option<&AtomicBool>,
    ) -> Result<ComputeTaskResponse, CloudError> {
        let start = Instant::now();

        // 0. Pre-execution cancellation check
        if cancellation_token.is_some_and(|c| c.load(Ordering::SeqCst)) {
            return Err(CloudError::OperationCancelled);
        }

        // 1. Validate TLS configuration
        self.tls_config.validate()?;

        // 2. Pre-flight consent authorization gate: fails closed if Local-Only Privacy Mode is active
        // or consent token is expired, revoked, mismatched, or over quota.
        self.consent_manager.verify_and_authorize(
            &req.token_id,
            req.modality,
            endpoint,
            req.payload.len() as u64,
            current_epoch_secs,
            privacy,
        )?;

        // 3. Pre-transmission cancellation check
        if cancellation_token.is_some_and(|c| c.load(Ordering::SeqCst)) {
            return Err(CloudError::OperationCancelled);
        }

        // 4. Remote network transmission
        let mut response = self.transport.dispatch(endpoint, &req)?;

        // 5. Mid-flight / post-transmission cancellation check
        if cancellation_token.is_some_and(|c| c.load(Ordering::SeqCst)) {
            return Err(CloudError::OperationCancelled);
        }

        // 6. Record consumed payload bytes in the ledger and update quota
        self.consent_manager.record_dispatch(
            &req.token_id,
            req.payload.len() as u64,
            current_epoch_secs,
        )?;

        response.latency_ms = start.elapsed().as_millis() as u64;
        Ok(response)
    }
}
