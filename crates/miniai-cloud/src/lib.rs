#![forbid(unsafe_code)]
//! # MiniAI Cloud (`miniai-cloud`)
//!
//! Consent-gated hybrid cloud execution, scoped `CloudConsentToken` lifecycle management,
//! cryptographic token fingerprinting, immutable append-only SQLite audit logging,
//! payload quota accounting, and instant local CPU fallback coordination.

pub mod dispatcher;
pub mod error;
pub mod fallback;
pub mod ledger;
pub mod manager;
pub mod token;
pub mod types;

pub use dispatcher::{
    ComputeTaskRequest, ComputeTaskResponse, RemoteDispatcher, RemoteWorkerTransport,
    SimulatedRemoteTransport, TlsConfig,
};
pub use error::CloudError;
pub use fallback::{DefaultLocalFallbackHandler, FallbackCoordinator, LocalFallbackHandler};
pub use ledger::ConsentLedger;
pub use manager::ConsentManager;
pub use miniai_core::CloudConsentToken;
pub use token::{TokenValidator, MAX_CONSENT_DURATION_SECS, MAX_PAYLOAD_CEILING_BYTES};
pub use types::{
    ConsentAction, ConsentAuditRecord, ConsentScope, ConsentStatus, ManagedConsentToken,
};
