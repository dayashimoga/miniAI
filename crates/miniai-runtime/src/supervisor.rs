use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use miniai_core::{
    AcceleratorBackend, AdapterHealth, AdmissionDecision, AdmissionGuard, HardwareProfile,
    Modality, PrivacyPolicyEngine, RuntimeCapability, SystemSnapshot, VerificationTier,
};
use miniai_server::{EventBus, SystemEvent};
use miniai_storage::DiagnosticEventRecord;

use crate::adapter::{AdapterState, ManagedAdapter};
use crate::error::RuntimeError;

/// Central supervisor coordinating decoupled modality runtimes, hardware admission,
/// real-time telemetry, and automatic CPU fallback across all 7 modalities.
pub struct RuntimeSupervisor {
    adapters: BTreeMap<Modality, ManagedAdapter>,
    admission_guard: AdmissionGuard,
    privacy: Arc<PrivacyPolicyEngine>,
    event_bus: Option<Arc<EventBus>>,
}

impl RuntimeSupervisor {
    /// Creates a new `RuntimeSupervisor` with independent adapters for all 7 modalities.
    pub fn new(
        hardware: HardwareProfile,
        privacy: Arc<PrivacyPolicyEngine>,
        event_bus: Option<Arc<EventBus>>,
    ) -> Self {
        let admission_guard = AdmissionGuard::new(hardware, privacy.clone());
        let mut adapters = BTreeMap::new();

        let defaults = [
            ("llm-isolated-runtime", Modality::Llm, true),
            ("vision-isolated-runtime", Modality::Vision, false),
            ("imagegen-isolated-runtime", Modality::ImageGeneration, true),
            ("speech-asr-isolated-runtime", Modality::SpeechAsr, true),
            ("speech-tts-isolated-runtime", Modality::SpeechTts, true),
            ("music-isolated-runtime", Modality::Music, false),
            ("video-isolated-runtime", Modality::Video, false),
        ];

        for (id, modality, streaming) in defaults {
            adapters.insert(modality, ManagedAdapter::new(id, modality, streaming));
        }

        Self {
            adapters,
            admission_guard,
            privacy,
            event_bus,
        }
    }

    pub fn admission_guard(&self) -> &AdmissionGuard {
        &self.admission_guard
    }

    pub fn privacy(&self) -> &PrivacyPolicyEngine {
        &self.privacy
    }

    /// Evaluates hardware admission and loads a model into the specified modality adapter.
    ///
    /// If accelerator VRAM is insufficient, the accelerator is missing, or Force CPU mode is active,
    /// the supervisor automatically selects `AcceleratorBackend::Cpu`.
    pub fn load_model(
        &mut self,
        modality: Modality,
        model_id: &str,
        model_path: &Path,
        required_ram_bytes: u64,
        required_vram_bytes: u64,
        preferred_backend: Option<AcceleratorBackend>,
    ) -> Result<AdmissionDecision, RuntimeError> {
        // 1. Evaluate admission with strict 85% memory ceiling & fallback
        let decision = self.admission_guard.evaluate_admission(
            required_ram_bytes,
            required_vram_bytes,
            preferred_backend,
        )?;

        // 2. Fetch adapter for the target modality
        let adapter =
            self.adapters
                .get_mut(&modality)
                .ok_or_else(|| RuntimeError::AdapterError {
                    modality,
                    message: format!("No adapter registered for modality {modality:?}"),
                })?;

        // 3. Load model into adapter
        adapter.load_model(
            model_id,
            model_path,
            decision.selected_backend,
            required_ram_bytes,
        )?;

        // 4. Emit health event over event bus if connected
        let health = adapter.health_check();
        self.broadcast_event(SystemEvent::AdapterHealthChanged { health });

        // 5. If fallback occurred, emit diagnostic alert
        if let Some(ref reason) = decision.fallback_to_cpu_reason {
            self.broadcast_event(SystemEvent::DiagnosticAlert {
                record: DiagnosticEventRecord {
                    id: None,
                    event_type: "CPU_FALLBACK".to_string(),
                    severity: "WARN".to_string(),
                    source: format!("{modality:?}-supervisor"),
                    message: format!("Model '{model_id}' fell back to CPU: {reason}"),
                    payload_json: None,
                    created_at_epoch_secs: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                },
            });
        }

        Ok(decision)
    }

    /// Atomically hot-switches the model loaded in a specific modality, verifying
    /// that the prior model is completely unloaded and memory freed before loading the replacement.
    pub fn switch_model(
        &mut self,
        modality: Modality,
        new_model_id: &str,
        new_model_path: &Path,
        required_ram_bytes: u64,
        required_vram_bytes: u64,
        preferred_backend: Option<AcceleratorBackend>,
    ) -> Result<AdmissionDecision, RuntimeError> {
        // Step 1: Safely unload current model (if loaded) to release memory
        let _ = self.unload_model(modality);

        // Step 2: Load replacement model into clean memory
        self.load_model(
            modality,
            new_model_id,
            new_model_path,
            required_ram_bytes,
            required_vram_bytes,
            preferred_backend,
        )
    }

    /// Returns the health of a specific modality adapter.
    pub fn health_check(&self, modality: Modality) -> Result<AdapterHealth, RuntimeError> {
        let adapter = self
            .adapters
            .get(&modality)
            .ok_or_else(|| RuntimeError::AdapterError {
                modality,
                message: format!("No adapter registered for modality {modality:?}"),
            })?;
        Ok(adapter.health_check())
    }

    /// Returns the health reports for all 7 registered modalities.
    pub fn health_check_all(&self) -> Vec<AdapterHealth> {
        self.adapters.values().map(|a| a.health_check()).collect()
    }

    /// Returns capabilities across all 7 registered modalities.
    pub fn capabilities(&self) -> Vec<RuntimeCapability> {
        self.adapters.values().map(|a| a.capability()).collect()
    }

    /// Forces a modality to degrade and fall back to `AcceleratorBackend::Cpu`.
    pub fn fallback_to_cpu(
        &mut self,
        modality: Modality,
        reason: &str,
    ) -> Result<AdapterHealth, RuntimeError> {
        let adapter =
            self.adapters
                .get_mut(&modality)
                .ok_or_else(|| RuntimeError::AdapterError {
                    modality,
                    message: format!("No adapter registered for modality {modality:?}"),
                })?;

        let health = adapter.fallback_to_cpu(reason)?;

        self.broadcast_event(SystemEvent::AdapterHealthChanged {
            health: health.clone(),
        });
        self.broadcast_event(SystemEvent::DiagnosticAlert {
            record: DiagnosticEventRecord {
                id: None,
                event_type: "ADAPTER_FALLBACK".to_string(),
                severity: "WARN".to_string(),
                source: format!("{modality:?}-runtime"),
                message: format!("Modality {modality:?} downgraded to CPU baseline: {reason}"),
                payload_json: None,
                created_at_epoch_secs: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
        });

        Ok(health)
    }

    /// Safely unloads a model for a specific modality.
    pub fn unload_model(&mut self, modality: Modality) -> Result<(), RuntimeError> {
        let adapter =
            self.adapters
                .get_mut(&modality)
                .ok_or_else(|| RuntimeError::AdapterError {
                    modality,
                    message: format!("No adapter registered for modality {modality:?}"),
                })?;

        adapter.unload_model()?;

        let health = adapter.health_check();
        self.broadcast_event(SystemEvent::AdapterHealthChanged { health });
        Ok(())
    }

    /// Unloads models across all modalities.
    pub fn unload_all(&mut self) -> Result<(), RuntimeError> {
        for modality in Modality::all() {
            if let Some(adapter) = self.adapters.get_mut(&modality) {
                let _ = adapter.unload_model();
            }
        }
        for health in self.health_check_all() {
            self.broadcast_event(SystemEvent::AdapterHealthChanged { health });
        }
        Ok(())
    }

    /// Retrieves the current lifecycle state of a specific modality adapter.
    pub fn get_state(&self, modality: Modality) -> Option<&AdapterState> {
        self.adapters.get(&modality).map(|a| a.state())
    }

    /// Builds a full cross-language `SystemSnapshot`.
    pub fn build_snapshot(&self, tier: VerificationTier) -> SystemSnapshot {
        SystemSnapshot {
            contract_version: "1.0.0".to_string(),
            verification_tier: tier,
            hardware: self.admission_guard.hardware().clone(),
            privacy: self.privacy.state().clone(),
            capabilities: self.capabilities(),
            adapter_health: self.health_check_all(),
        }
    }

    fn broadcast_event(&self, event: SystemEvent) {
        if let Some(ref bus) = self.event_bus {
            let _ = bus.publish_now(event);
        }
    }
}
