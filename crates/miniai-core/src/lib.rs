#![forbid(unsafe_code)]
//! # MiniAI Core (`miniai-core`)
//!
//! Foundational contracts, privacy/consent enforcement, CPU-first hardware detection,
//! SHA-256 artifact integrity verification, and independently pluggable modality
//! runtime adapter registry for the MiniAI platform.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

/// Canonical AI modalities supported by MiniAI via independent runtime adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Llm,
    Vision,
    ImageGeneration,
    SpeechAsr,
    SpeechTts,
    Music,
    Video,
}

impl Modality {
    /// Returns all supported modalities in canonical order.
    pub const fn all() -> [Modality; 7] {
        [
            Modality::Llm,
            Modality::Vision,
            Modality::ImageGeneration,
            Modality::SpeechAsr,
            Modality::SpeechTts,
            Modality::Music,
            Modality::Video,
        ]
    }

    /// Returns a human-readable label for the modality.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Modality::Llm => "LLM Text & Reasoning",
            Modality::Vision => "Vision Understanding",
            Modality::ImageGeneration => "Image Generation & Editing",
            Modality::SpeechAsr => "Speech Recognition (ASR)",
            Modality::SpeechTts => "Text-to-Speech (TTS)",
            Modality::Music => "Music & Audio Synthesis",
            Modality::Video => "Video Understanding & Generation",
        }
    }
}

/// Hardware execution backends with guaranteed CPU-first baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorBackend {
    Cpu,
    Cuda,
    Vulkan,
    Metal,
    DirectMl,
    Rocm,
    CoreMl,
    Nnapi,
}

impl AcceleratorBackend {
    /// Returns true if this backend is the universal CPU execution path.
    pub const fn is_cpu(&self) -> bool {
        matches!(self, AcceleratorBackend::Cpu)
    }

    /// Returns a human-readable label for the backend.
    pub const fn display_name(&self) -> &'static str {
        match self {
            AcceleratorBackend::Cpu => "CPU (Universal Baseline)",
            AcceleratorBackend::Cuda => "NVIDIA CUDA",
            AcceleratorBackend::Vulkan => "Vulkan Compute",
            AcceleratorBackend::Metal => "Apple Metal",
            AcceleratorBackend::DirectMl => "DirectML",
            AcceleratorBackend::Rocm => "AMD ROCm",
            AcceleratorBackend::CoreMl => "Apple CoreML",
            AcceleratorBackend::Nnapi => "Android NNAPI",
        }
    }
}

/// Honest verification provenance classification required by the Global Engineering Contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationTier {
    ProvenOnHost,
    VerifiedInCi,
    HardwareDependent,
}

/// Domain errors produced across MiniAI core services.
#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "code", content = "details", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MiniAiError {
    #[error("Outbound cloud execution is blocked while Local-Only Privacy Mode is active")]
    PrivacyModeBlocked,

    #[error("Explicit user CloudConsentToken is required for remote compute execution")]
    ConsentRequired,

    #[error(
        "CloudConsentToken has expired (current_epoch={current_epoch}, expires_at={expires_at})"
    )]
    ConsentExpired { current_epoch: u64, expires_at: u64 },

    #[error("CloudConsentToken scope mismatch: {reason}")]
    ConsentScopeMismatch { reason: String },

    #[error("Artifact SHA-256 mismatch: expected {expected}, actual {actual}")]
    IntegrityMismatch { expected: String, actual: String },

    #[error("Invalid SHA-256 hex digest format: {digest}")]
    InvalidDigestFormat { digest: String },

    #[error("Unsafe or traversal path rejected: {path}")]
    UnsafePathRejected { path: String },

    #[error("Insufficient memory for model: required {required_bytes} bytes, safe limit {safe_limit_bytes} bytes")]
    InsufficientMemory {
        required_bytes: u64,
        safe_limit_bytes: u64,
    },

    #[error("Duplicate modality adapter ID registered: {adapter_id}")]
    DuplicateAdapter { adapter_id: String },

    #[error("Modality adapter not found: {adapter_id}")]
    AdapterNotFound { adapter_id: String },

    #[error("Contract serialization error: {message}")]
    SerializationError { message: String },
}

pub mod hardware;

pub use hardware::{
    probe_host_hardware, AdmissionDecision, AdmissionGuard, CpuFeatures, DeviceGovernor, GpuDevice,
    HardwareProfile, PowerGovernorProfile, PowerState, ThermalState,
};

/// Hardware discovery service.
pub struct HardwareDetector;

impl HardwareDetector {
    /// Detects the current host profile with guaranteed `AcceleratorBackend::Cpu` baseline
    /// and dynamic SIMD / RAM / VRAM probing.
    pub fn detect_host_profile() -> HardwareProfile {
        HardwareProfile::probe()
    }
}

/// Scoped, time-bounded user consent token required for any hybrid cloud compute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudConsentToken {
    pub token_id: String,
    pub modality: Modality,
    pub target_endpoint: String,
    pub max_payload_bytes: u64,
    pub issued_at_epoch_secs: u64,
    pub expires_at_epoch_secs: u64,
    pub user_acknowledged: bool,
}

/// Serializable state of the MiniAI Privacy & Consent Engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyPolicyState {
    pub local_only_mode: bool,
    pub force_cpu_only: bool,
    pub active_consent: Option<CloudConsentToken>,
}

impl Default for PrivacyPolicyState {
    fn default() -> Self {
        Self {
            local_only_mode: true,
            force_cpu_only: false,
            active_consent: None,
        }
    }
}

/// Enforces `LocalOnlyPrivacyMode` (enabled by default) and validates explicit cloud consent tokens.
#[derive(Debug, Clone, Default)]
pub struct PrivacyPolicyEngine {
    state: PrivacyPolicyState,
}

impl PrivacyPolicyEngine {
    /// Creates a new `PrivacyPolicyEngine` with `local_only_mode = true` by default.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an immutable reference to the current privacy state.
    pub fn state(&self) -> &PrivacyPolicyState {
        &self.state
    }

    /// Returns `true` if Local-Only Privacy Mode is active.
    pub fn is_local_only(&self) -> bool {
        self.state.local_only_mode
    }

    /// Enables or disables `LocalOnlyPrivacyMode`. Enabling it immediately revokes any active cloud consent token.
    pub fn set_local_only_mode(&mut self, enabled: bool) {
        self.state.local_only_mode = enabled;
        if enabled {
            self.state.active_consent = None;
        }
    }

    /// Sets whether execution must strictly use CPU even if accelerators are present.
    pub fn set_force_cpu_only(&mut self, force_cpu: bool) {
        self.state.force_cpu_only = force_cpu;
    }

    /// Grants an explicit `CloudConsentToken` if and only if `local_only_mode` is disabled and the token is valid.
    pub fn grant_cloud_consent(&mut self, token: CloudConsentToken) -> Result<(), MiniAiError> {
        if self.state.local_only_mode {
            return Err(MiniAiError::PrivacyModeBlocked);
        }
        if !token.user_acknowledged || token.token_id.trim().is_empty() {
            return Err(MiniAiError::ConsentScopeMismatch {
                reason: "Token must have a non-empty token_id and explicit user_acknowledged=true"
                    .to_string(),
            });
        }
        if !token.target_endpoint.starts_with("https://") {
            return Err(MiniAiError::ConsentScopeMismatch {
                reason: "Cloud target_endpoint must use https://".to_string(),
            });
        }
        if token.expires_at_epoch_secs <= token.issued_at_epoch_secs {
            return Err(MiniAiError::ConsentExpired {
                current_epoch: token.issued_at_epoch_secs,
                expires_at: token.expires_at_epoch_secs,
            });
        }
        self.state.active_consent = Some(token);
        Ok(())
    }

    /// Immediately revokes any active `CloudConsentToken`.
    pub fn revoke_cloud_consent(&mut self) {
        self.state.active_consent = None;
    }

    /// Verifies whether a remote cloud execution request is permitted under current privacy & consent rules.
    pub fn authorize_cloud_execution(
        &self,
        modality: Modality,
        endpoint: &str,
        payload_bytes: u64,
        current_epoch_secs: u64,
    ) -> Result<(), MiniAiError> {
        if self.state.local_only_mode {
            return Err(MiniAiError::PrivacyModeBlocked);
        }
        let token = self
            .state
            .active_consent
            .as_ref()
            .ok_or(MiniAiError::ConsentRequired)?;

        if !token.user_acknowledged {
            return Err(MiniAiError::ConsentRequired);
        }
        if current_epoch_secs >= token.expires_at_epoch_secs {
            return Err(MiniAiError::ConsentExpired {
                current_epoch: current_epoch_secs,
                expires_at: token.expires_at_epoch_secs,
            });
        }
        if token.modality != modality {
            return Err(MiniAiError::ConsentScopeMismatch {
                reason: format!(
                    "Token authorized for {:?}, but {:?} was requested",
                    token.modality, modality
                ),
            });
        }
        if token.target_endpoint != endpoint {
            return Err(MiniAiError::ConsentScopeMismatch {
                reason: format!(
                    "Token authorized for endpoint '{}', but '{}' was requested",
                    token.target_endpoint, endpoint
                ),
            });
        }
        if payload_bytes > token.max_payload_bytes {
            return Err(MiniAiError::ConsentScopeMismatch {
                reason: format!(
                    "Payload size {} exceeds consented limit {} bytes",
                    payload_bytes, token.max_payload_bytes
                ),
            });
        }
        Ok(())
    }
}

/// Computes the lowercase 64-character hexadecimal SHA-256 digest of `data`.
pub fn compute_sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut hex_out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut hex_out, "{byte:02x}");
    }
    hex_out
}

/// Verifies that `data` matches the expected 64-character hex SHA-256 digest.
pub fn verify_sha256_digest(data: &[u8], expected_hex: &str) -> Result<(), MiniAiError> {
    let normalized = expected_hex.trim().to_ascii_lowercase();
    if normalized.len() != 64 || !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(MiniAiError::InvalidDigestFormat {
            digest: expected_hex.to_string(),
        });
    }
    let actual = compute_sha256_hex(data);
    if actual != normalized {
        return Err(MiniAiError::IntegrityMismatch {
            expected: normalized,
            actual,
        });
    }
    Ok(())
}

/// Validates that a relative path within the model or dataset registry contains no traversal or root escapes.
pub fn validate_safe_relative_path(path_str: &str) -> Result<String, MiniAiError> {
    let trimmed = path_str.trim();
    if trimmed.is_empty()
        || trimmed.contains('\0')
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.contains(':')
    {
        return Err(MiniAiError::UnsafePathRejected {
            path: path_str.to_string(),
        });
    }

    let normalized = trimmed.replace('\\', "/");
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(MiniAiError::UnsafePathRejected {
                path: path_str.to_string(),
            });
        }
    }
    Ok(normalized)
}

/// Static capability descriptor for a registered modality runtime adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapability {
    pub adapter_id: String,
    pub modality: Modality,
    pub backends: Vec<AcceleratorBackend>,
    pub supports_streaming: bool,
    pub supports_offline: bool,
    pub isolated_process: bool,
}

/// Live health status of a modality runtime adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterHealth {
    pub adapter_id: String,
    pub modality: Modality,
    pub ready: bool,
    pub active_backend: AcceleratorBackend,
    pub status_message: String,
}

/// Trait implemented by every independent modality runtime adapter.
pub trait ModalityAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn modality(&self) -> Modality;
    fn supported_backends(&self) -> Vec<AcceleratorBackend>;
    fn capability(&self) -> RuntimeCapability;
    fn health_check(&self) -> Result<AdapterHealth, MiniAiError>;
}

/// Baseline CPU-first adapter for a single modality, proving runtime decoupling.
#[derive(Debug, Clone)]
pub struct CpuBaselineModalityAdapter {
    adapter_id: String,
    modality: Modality,
    supports_streaming: bool,
}

impl CpuBaselineModalityAdapter {
    pub fn new(
        adapter_id: impl Into<String>,
        modality: Modality,
        supports_streaming: bool,
    ) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            modality,
            supports_streaming,
        }
    }
}

impl ModalityAdapter for CpuBaselineModalityAdapter {
    fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    fn modality(&self) -> Modality {
        self.modality
    }

    fn supported_backends(&self) -> Vec<AcceleratorBackend> {
        vec![AcceleratorBackend::Cpu]
    }

    fn capability(&self) -> RuntimeCapability {
        RuntimeCapability {
            adapter_id: self.adapter_id.clone(),
            modality: self.modality,
            backends: vec![AcceleratorBackend::Cpu],
            supports_streaming: self.supports_streaming,
            supports_offline: true,
            isolated_process: true,
        }
    }

    fn health_check(&self) -> Result<AdapterHealth, MiniAiError> {
        Ok(AdapterHealth {
            adapter_id: self.adapter_id.clone(),
            modality: self.modality,
            ready: true,
            active_backend: AcceleratorBackend::Cpu,
            status_message: format!(
                "{} adapter ready on isolated CPU runtime",
                self.modality.display_name()
            ),
        })
    }
}

/// Complete serializable snapshot of MiniAI core status shared across Rust, Flutter, and Python.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub contract_version: String,
    pub verification_tier: VerificationTier,
    pub hardware: HardwareProfile,
    pub privacy: PrivacyPolicyState,
    pub capabilities: Vec<RuntimeCapability>,
    pub adapter_health: Vec<AdapterHealth>,
}

/// Registry managing independently pluggable `ModalityAdapter` instances.
#[derive(Default)]
pub struct RuntimeRegistry {
    adapters: BTreeMap<String, Box<dyn ModalityAdapter>>,
}

impl RuntimeRegistry {
    /// Creates an empty `RuntimeRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry populated with independent CPU-first baseline adapters for all 7 modalities.
    pub fn with_default_cpu_adapters() -> Result<Self, MiniAiError> {
        let mut registry = Self::new();
        let defaults = [
            ("llm-cpu-adapter", Modality::Llm, true),
            ("vision-cpu-adapter", Modality::Vision, false),
            ("imagegen-cpu-adapter", Modality::ImageGeneration, true),
            ("speech-asr-cpu-adapter", Modality::SpeechAsr, true),
            ("speech-tts-cpu-adapter", Modality::SpeechTts, true),
            ("music-cpu-adapter", Modality::Music, false),
            ("video-cpu-adapter", Modality::Video, false),
        ];
        for (id, modality, streaming) in defaults {
            registry.register(Box::new(CpuBaselineModalityAdapter::new(
                id, modality, streaming,
            )))?;
        }
        Ok(registry)
    }

    /// Registers an independent modality adapter. Rejects duplicate adapter IDs.
    pub fn register(&mut self, adapter: Box<dyn ModalityAdapter>) -> Result<(), MiniAiError> {
        let id = adapter.adapter_id().to_string();
        if self.adapters.contains_key(&id) {
            return Err(MiniAiError::DuplicateAdapter { adapter_id: id });
        }
        self.adapters.insert(id, adapter);
        Ok(())
    }

    /// Returns the number of registered modality adapters.
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    /// Returns true if no adapters are registered.
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    /// Lists capabilities for all registered adapters.
    pub fn capabilities(&self) -> Vec<RuntimeCapability> {
        self.adapters.values().map(|a| a.capability()).collect()
    }

    /// Lists adapters matching a specific `Modality`.
    pub fn capabilities_for_modality(&self, modality: Modality) -> Vec<RuntimeCapability> {
        self.adapters
            .values()
            .filter(|a| a.modality() == modality)
            .map(|a| a.capability())
            .collect()
    }

    /// Runs health checks across all registered adapters.
    pub fn health_report(&self) -> Result<Vec<AdapterHealth>, MiniAiError> {
        let mut out = Vec::with_capacity(self.adapters.len());
        for adapter in self.adapters.values() {
            out.push(adapter.health_check()?);
        }
        Ok(out)
    }

    /// Queries health of a specific adapter by ID.
    pub fn check_adapter(&self, adapter_id: &str) -> Result<AdapterHealth, MiniAiError> {
        let adapter =
            self.adapters
                .get(adapter_id)
                .ok_or_else(|| MiniAiError::AdapterNotFound {
                    adapter_id: adapter_id.to_string(),
                })?;
        adapter.health_check()
    }

    /// Builds a complete `SystemSnapshot` for cross-language consumption.
    pub fn build_snapshot(
        &self,
        hardware: HardwareProfile,
        privacy: PrivacyPolicyState,
        verification_tier: VerificationTier,
    ) -> Result<SystemSnapshot, MiniAiError> {
        Ok(SystemSnapshot {
            contract_version: "1.0.0".to_string(),
            verification_tier,
            hardware,
            privacy,
            capabilities: self.capabilities(),
            adapter_health: self.health_report()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_all_modalities_and_display_names() {
        let all = Modality::all();
        assert_eq!(all.len(), 7);
        for m in all {
            assert!(!m.display_name().is_empty());
        }
    }

    #[test]
    fn test_accelerator_backend_properties() {
        assert!(AcceleratorBackend::Cpu.is_cpu());
        assert!(!AcceleratorBackend::Cuda.is_cpu());
        assert_eq!(
            AcceleratorBackend::Cpu.display_name(),
            "CPU (Universal Baseline)"
        );
        assert_eq!(AcceleratorBackend::Vulkan.display_name(), "Vulkan Compute");
        assert_eq!(AcceleratorBackend::Metal.display_name(), "Apple Metal");
        assert_eq!(AcceleratorBackend::DirectMl.display_name(), "DirectML");
        assert_eq!(AcceleratorBackend::Rocm.display_name(), "AMD ROCm");
        assert_eq!(AcceleratorBackend::CoreMl.display_name(), "Apple CoreML");
        assert_eq!(AcceleratorBackend::Nnapi.display_name(), "Android NNAPI");
        assert_eq!(AcceleratorBackend::Cuda.display_name(), "NVIDIA CUDA");
    }

    #[test]
    fn test_hardware_admission_and_backend_fallback() {
        let mut profile = HardwareDetector::detect_host_profile();
        assert!(profile.logical_cpu_cores >= 1);
        assert!(profile.detected_backends.contains(&AcceleratorBackend::Cpu));

        let limit = profile.safe_memory_limit_bytes();
        assert!(limit > 0);
        assert!(profile.can_admit_model(1024 * 1024 * 10).is_ok());
        assert!(matches!(
            profile.can_admit_model(0),
            Err(MiniAiError::InsufficientMemory { .. })
        ));
        assert!(matches!(
            profile.can_admit_model(limit + 1024),
            Err(MiniAiError::InsufficientMemory { .. })
        ));

        // When ROCm (absent) is preferred, falls back to CPU
        profile
            .detected_backends
            .retain(|b| *b != AcceleratorBackend::Rocm);
        assert_eq!(
            profile.select_execution_backend(Some(AcceleratorBackend::Rocm), false),
            AcceleratorBackend::Cpu
        );

        // When an accelerator is detected, selects it unless force_cpu is true
        if !profile
            .detected_backends
            .contains(&AcceleratorBackend::Vulkan)
        {
            profile.detected_backends.push(AcceleratorBackend::Vulkan);
        }
        assert_eq!(
            profile.select_execution_backend(Some(AcceleratorBackend::Vulkan), false),
            AcceleratorBackend::Vulkan
        );
        assert_eq!(
            profile.select_execution_backend(Some(AcceleratorBackend::Vulkan), true),
            AcceleratorBackend::Cpu
        );
    }

    #[test]
    fn test_admission_guard_evaluates_memory_ceiling_and_cpu_fallback() {
        let profile = HardwareProfile {
            os_family: "linux".to_string(),
            arch: "x86_64".to_string(),
            logical_cpu_cores: 16,
            total_memory_bytes: 32_000_000_000,
            available_memory_bytes: 10_000_000_000, // 10 GB avail -> 8.5 GB safe limit
            detected_backends: vec![AcceleratorBackend::Cpu, AcceleratorBackend::Cuda],
            cpu_features: CpuFeatures::detect(),
            gpus: vec![GpuDevice {
                name: "NVIDIA RTX 4090".to_string(),
                backend: AcceleratorBackend::Cuda,
                total_vram_bytes: 24_000_000_000,
                available_vram_bytes: 4_000_000_000, // 4 GB avail -> 3.4 GB safe limit
            }],
        };

        let mut privacy = PrivacyPolicyEngine::new();
        let guard = AdmissionGuard::new(profile.clone(), Arc::new(privacy.clone()));

        let safe_ram = guard.safe_ram_limit_bytes();
        assert_eq!(safe_ram, 8_500_000_000); // 85% of 10GB

        // 1. Fits within VRAM and RAM on CUDA
        let decision = guard
            .evaluate_admission(2_000_000_000, 3_000_000_000, Some(AcceleratorBackend::Cuda))
            .expect("should admit on CUDA");
        assert!(decision.admitted);
        assert_eq!(decision.selected_backend, AcceleratorBackend::Cuda);
        assert!(decision.fallback_to_cpu_reason.is_none());

        // 2. Exceeds VRAM safe limit (3.5 GB > 3.4 GB) -> Fallback to CPU
        let fallback_decision = guard
            .evaluate_admission(2_000_000_000, 3_500_000_000, Some(AcceleratorBackend::Cuda))
            .expect("should fall back to CPU");
        assert!(fallback_decision.admitted);
        assert_eq!(fallback_decision.selected_backend, AcceleratorBackend::Cpu);
        assert!(fallback_decision.fallback_to_cpu_reason.is_some());

        // 3. Accelerator not present on host -> Fallback to CPU
        let no_gpu_decision = guard
            .evaluate_admission(
                1_000_000_000,
                1_000_000_000,
                Some(AcceleratorBackend::Metal),
            )
            .expect("should fall back to CPU for missing backend");
        assert!(no_gpu_decision.admitted);
        assert_eq!(no_gpu_decision.selected_backend, AcceleratorBackend::Cpu);
        assert!(no_gpu_decision.fallback_to_cpu_reason.is_some());

        // 4. Force CPU mode active -> Restricts to CPU even when GPU has plenty of VRAM
        privacy.set_force_cpu_only(true);
        let guard_force_cpu = AdmissionGuard::new(profile.clone(), Arc::new(privacy.clone()));
        let force_cpu_decision = guard_force_cpu
            .evaluate_admission(1_000_000_000, 1_000_000_000, Some(AcceleratorBackend::Cuda))
            .expect("admit on CPU due to force CPU policy");
        assert_eq!(force_cpu_decision.selected_backend, AcceleratorBackend::Cpu);
        assert!(force_cpu_decision.fallback_to_cpu_reason.is_some());

        // 5. Exceeds 85% RAM safe limit -> InsufficientMemory error
        privacy.set_force_cpu_only(false);
        let guard_oom = AdmissionGuard::new(profile.clone(), Arc::new(privacy));
        let err = guard_oom
            .evaluate_admission(9_000_000_000, 0, Some(AcceleratorBackend::Cpu))
            .expect_err("should reject model exceeding 85% RAM limit");
        match err {
            MiniAiError::InsufficientMemory {
                required_bytes,
                safe_limit_bytes,
            } => {
                assert_eq!(required_bytes, 9_000_000_000);
                assert_eq!(safe_limit_bytes, 8_500_000_000);
            }
            other => panic!("Expected InsufficientMemory, got {other:?}"),
        }
    }

    #[test]
    fn test_cpu_features_simd_detection() {
        let features = CpuFeatures::detect();
        let summary = features.enabled_features();

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // On modern x86_64, at least SSE4.2 is enabled
            if is_x86_feature_detected!("sse4.2") {
                assert!(features.sse4_2);
                assert!(summary.contains(&"SSE4.2"));
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            assert!(features.neon);
            assert!(summary.contains(&"NEON"));
        }
    }

    #[test]
    fn test_privacy_policy_local_only_and_consent_enforcement() {
        let mut engine = PrivacyPolicyEngine::new();
        assert!(engine.state().local_only_mode);

        // Blocked while Local-Only Privacy Mode is active
        let err = engine
            .authorize_cloud_execution(Modality::Llm, "https://gpu.miniai.local", 1024, 1000)
            .unwrap_err();
        assert_eq!(err, MiniAiError::PrivacyModeBlocked);

        let sample_token = CloudConsentToken {
            token_id: "consent-001".to_string(),
            modality: Modality::Llm,
            target_endpoint: "https://gpu.miniai.local".to_string(),
            max_payload_bytes: 4096,
            issued_at_epoch_secs: 1000,
            expires_at_epoch_secs: 2000,
            user_acknowledged: true,
        };

        // Cannot grant consent while local_only_mode is true
        assert_eq!(
            engine.grant_cloud_consent(sample_token.clone()),
            Err(MiniAiError::PrivacyModeBlocked)
        );

        // Disable local_only_mode; still requires consent token
        engine.set_local_only_mode(false);
        assert_eq!(
            engine.authorize_cloud_execution(Modality::Llm, "https://gpu.miniai.local", 1024, 1200),
            Err(MiniAiError::ConsentRequired)
        );

        // Reject invalid tokens (unacknowledged, non-https, or invalid expiry)
        let mut unacked = sample_token.clone();
        unacked.user_acknowledged = false;
        assert!(matches!(
            engine.grant_cloud_consent(unacked),
            Err(MiniAiError::ConsentScopeMismatch { .. })
        ));

        let mut insecure = sample_token.clone();
        insecure.target_endpoint = "http://insecure.local".to_string();
        assert!(matches!(
            engine.grant_cloud_consent(insecure),
            Err(MiniAiError::ConsentScopeMismatch { .. })
        ));

        let mut bad_expiry = sample_token.clone();
        bad_expiry.expires_at_epoch_secs = 900;
        assert!(matches!(
            engine.grant_cloud_consent(bad_expiry),
            Err(MiniAiError::ConsentExpired { .. })
        ));

        // Grant valid token and verify authorization checks
        engine.grant_cloud_consent(sample_token).unwrap();
        assert!(engine
            .authorize_cloud_execution(Modality::Llm, "https://gpu.miniai.local", 2048, 1500)
            .is_ok());

        // Mismatched modality
        assert!(matches!(
            engine.authorize_cloud_execution(
                Modality::Vision,
                "https://gpu.miniai.local",
                2048,
                1500
            ),
            Err(MiniAiError::ConsentScopeMismatch { .. })
        ));

        // Mismatched endpoint
        assert!(matches!(
            engine.authorize_cloud_execution(
                Modality::Llm,
                "https://other.miniai.local",
                2048,
                1500
            ),
            Err(MiniAiError::ConsentScopeMismatch { .. })
        ));

        // Payload exceeds consented max_payload_bytes
        assert!(matches!(
            engine.authorize_cloud_execution(Modality::Llm, "https://gpu.miniai.local", 8192, 1500),
            Err(MiniAiError::ConsentScopeMismatch { .. })
        ));

        // Expired timestamp
        assert!(matches!(
            engine.authorize_cloud_execution(Modality::Llm, "https://gpu.miniai.local", 1024, 2000),
            Err(MiniAiError::ConsentExpired { .. })
        ));

        // Re-enabling local_only_mode revokes active consent immediately
        engine.set_local_only_mode(true);
        assert!(engine.state().active_consent.is_none());
    }

    #[test]
    fn test_sha256_verification_and_path_sanitization() {
        let payload = b"miniai-verified-model-weights-v1";
        let digest = compute_sha256_hex(payload);
        assert_eq!(digest.len(), 64);
        assert!(verify_sha256_digest(payload, &digest).is_ok());

        // Tampered payload rejected
        assert!(matches!(
            verify_sha256_digest(b"tampered-payload", &digest),
            Err(MiniAiError::IntegrityMismatch { .. })
        ));

        // Malformed hex digest rejected
        assert!(matches!(
            verify_sha256_digest(payload, "not-a-valid-sha256"),
            Err(MiniAiError::InvalidDigestFormat { .. })
        ));

        // Path validation
        assert_eq!(
            validate_safe_relative_path("models/llm/tiny-q4.gguf").unwrap(),
            "models/llm/tiny-q4.gguf"
        );
        for unsafe_path in [
            "",
            "../etc/passwd",
            "models/../../secret.key",
            "/abs/path.gguf",
            "\\unc\\share\\model.gguf",
            "C:\\Windows\\System32\\cmd.exe",
            "models/./nested/file.onnx",
        ] {
            assert!(
                matches!(
                    validate_safe_relative_path(unsafe_path),
                    Err(MiniAiError::UnsafePathRejected { .. })
                ),
                "Expected path '{unsafe_path}' to be rejected"
            );
        }
    }

    #[test]
    fn test_runtime_registry_and_snapshot() {
        let mut registry = RuntimeRegistry::with_default_cpu_adapters().unwrap();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 7);

        for modality in Modality::all() {
            let caps = registry.capabilities_for_modality(modality);
            assert_eq!(caps.len(), 1);
            assert!(caps[0].supports_offline);
            assert!(caps[0].isolated_process);
        }

        // Duplicate registration rejected
        let dup = Box::new(CpuBaselineModalityAdapter::new(
            "llm-cpu-adapter",
            Modality::Llm,
            true,
        ));
        assert!(matches!(
            registry.register(dup),
            Err(MiniAiError::DuplicateAdapter { .. })
        ));

        // Unknown adapter lookup fails cleanly
        assert!(matches!(
            registry.check_adapter("non-existent-adapter"),
            Err(MiniAiError::AdapterNotFound { .. })
        ));

        // Build full snapshot and verify JSON round-trip
        let mut policy = PrivacyPolicyEngine::new();
        policy.set_force_cpu_only(true);
        let snapshot = registry
            .build_snapshot(
                HardwareDetector::detect_host_profile(),
                policy.state().clone(),
                VerificationTier::ProvenOnHost,
            )
            .unwrap();

        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let decoded: SystemSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, decoded);
    }
}
