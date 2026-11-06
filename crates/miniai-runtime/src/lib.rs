#![forbid(unsafe_code)]

pub mod adapter;
pub mod error;
pub mod supervisor;

pub use adapter::{AdapterState, ManagedAdapter};
pub use error::RuntimeError;
pub use supervisor::RuntimeSupervisor;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use miniai_core::{
        AcceleratorBackend, CpuFeatures, GpuDevice, HardwareProfile, MiniAiError, Modality,
        PrivacyPolicyEngine, VerificationTier,
    };
    use miniai_server::EventBus;

    use super::*;

    fn create_test_hardware_profile(
        available_ram: u64,
        has_cuda: bool,
        available_vram: u64,
    ) -> HardwareProfile {
        let mut backends = vec![AcceleratorBackend::Cpu];
        let mut gpus = Vec::new();

        if has_cuda {
            backends.push(AcceleratorBackend::Cuda);
            gpus.push(GpuDevice {
                name: "Test NVIDIA GPU".to_string(),
                backend: AcceleratorBackend::Cuda,
                total_vram_bytes: available_vram * 2,
                available_vram_bytes: available_vram,
            });
        }

        HardwareProfile {
            os_family: "linux".to_string(),
            arch: "x86_64".to_string(),
            logical_cpu_cores: 8,
            total_memory_bytes: available_ram * 2,
            available_memory_bytes: available_ram,
            detected_backends: backends,
            cpu_features: CpuFeatures::detect(),
            gpus,
        }
    }

    #[test]
    fn test_supervisor_initializes_all_seven_modalities_in_unloaded_cpu_ready_state() {
        let hardware = create_test_hardware_profile(10_000_000_000, false, 0);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let supervisor = RuntimeSupervisor::new(hardware, privacy, None);

        let capabilities = supervisor.capabilities();
        assert_eq!(capabilities.len(), 7);

        let health_reports = supervisor.health_check_all();
        assert_eq!(health_reports.len(), 7);

        for health in health_reports {
            assert!(health.ready);
            assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
            assert!(health.status_message.contains("ready"));
        }

        for modality in Modality::all() {
            assert_eq!(
                supervisor.get_state(modality),
                Some(&AdapterState::Unloaded)
            );
        }
    }

    #[test]
    fn test_supervisor_loads_model_on_accelerator_when_resources_sufficient() {
        let hardware = create_test_hardware_profile(10_000_000_000, true, 8_000_000_000);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let event_bus = Arc::new(EventBus::new(32, 16));
        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, Some(event_bus));

        let model_path = PathBuf::from("/models/llama-3.2-1b.gguf");
        let decision = supervisor
            .load_model(
                Modality::Llm,
                "llama-3.2-1b",
                &model_path,
                2_000_000_000,
                3_000_000_000,
                Some(AcceleratorBackend::Cuda),
            )
            .expect("load should succeed on CUDA");

        assert!(decision.admitted);
        assert_eq!(decision.selected_backend, AcceleratorBackend::Cuda);
        assert!(decision.fallback_to_cpu_reason.is_none());

        let health = supervisor.health_check(Modality::Llm).unwrap();
        assert!(health.ready);
        assert_eq!(health.active_backend, AcceleratorBackend::Cuda);
        assert!(health.status_message.contains("llama-3.2-1b"));
    }

    #[test]
    fn test_supervisor_automatic_cpu_fallback_when_accelerator_vram_exceeds_ceiling() {
        // Available VRAM is 4 GB -> 85% ceiling is 3.4 GB
        let hardware = create_test_hardware_profile(10_000_000_000, true, 4_000_000_000);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let event_bus = Arc::new(EventBus::new(32, 16));
        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, Some(event_bus));

        let model_path = PathBuf::from("/models/vision-large.onnx");
        // Model requests 3.8 GB VRAM > 3.4 GB ceiling
        let decision = supervisor
            .load_model(
                Modality::Vision,
                "vision-large",
                &model_path,
                2_000_000_000,
                3_800_000_000,
                Some(AcceleratorBackend::Cuda),
            )
            .expect("should admit with CPU fallback");

        assert!(decision.admitted);
        assert_eq!(decision.selected_backend, AcceleratorBackend::Cpu);
        assert!(decision.fallback_to_cpu_reason.is_some());

        let health = supervisor.health_check(Modality::Vision).unwrap();
        assert!(health.ready);
        assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
    }

    #[test]
    fn test_supervisor_automatic_cpu_fallback_when_accelerator_absent() {
        // Host has NO CUDA
        let hardware = create_test_hardware_profile(10_000_000_000, false, 0);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, None);

        let model_path = PathBuf::from("/models/tts-model.bin");
        let decision = supervisor
            .load_model(
                Modality::SpeechTts,
                "tts-model",
                &model_path,
                500_000_000,
                1_000_000_000,
                Some(AcceleratorBackend::Cuda),
            )
            .expect("should admit with CPU fallback when accelerator missing");

        assert_eq!(decision.selected_backend, AcceleratorBackend::Cpu);
        assert!(decision.fallback_to_cpu_reason.is_some());
    }

    #[test]
    fn test_supervisor_automatic_cpu_fallback_when_force_cpu_active() {
        let hardware = create_test_hardware_profile(10_000_000_000, true, 16_000_000_000);
        let mut privacy_engine = PrivacyPolicyEngine::default();
        privacy_engine.set_force_cpu_only(true);
        let privacy = Arc::new(privacy_engine);
        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, None);

        let model_path = PathBuf::from("/models/asr-model.bin");
        let decision = supervisor
            .load_model(
                Modality::SpeechAsr,
                "asr-model",
                &model_path,
                500_000_000,
                1_000_000_000,
                Some(AcceleratorBackend::Cuda),
            )
            .expect("force CPU policy admitted on CPU");

        assert_eq!(decision.selected_backend, AcceleratorBackend::Cpu);
        assert!(decision.fallback_to_cpu_reason.is_some());
    }

    #[test]
    fn test_supervisor_rejects_admission_when_ram_exceeds_85_percent_limit() {
        // 10 GB available RAM -> 8.5 GB safe limit
        let hardware = create_test_hardware_profile(10_000_000_000, false, 0);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, None);

        let model_path = PathBuf::from("/models/huge-model.gguf");
        // Requests 9 GB RAM > 8.5 GB safe limit
        let err = supervisor
            .load_model(
                Modality::Llm,
                "huge-model",
                &model_path,
                9_000_000_000,
                0,
                Some(AcceleratorBackend::Cpu),
            )
            .expect_err("should reject admission exceeding 85% RAM ceiling");

        match err {
            RuntimeError::CoreError(MiniAiError::InsufficientMemory {
                required_bytes,
                safe_limit_bytes,
            }) => {
                assert_eq!(required_bytes, 9_000_000_000);
                assert_eq!(safe_limit_bytes, 8_500_000_000);
            }
            other => panic!("Expected InsufficientMemory, got {other:?}"),
        }
    }

    #[test]
    fn test_supervisor_decouples_modalities_failure_in_one_does_not_affect_others() {
        let hardware = create_test_hardware_profile(10_000_000_000, true, 8_000_000_000);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, None);

        // Load LLM
        let p1 = PathBuf::from("/models/llm.gguf");
        supervisor
            .load_model(
                Modality::Llm,
                "llm",
                &p1,
                1_000_000_000,
                2_000_000_000,
                Some(AcceleratorBackend::Cuda),
            )
            .unwrap();

        // Load Music
        let p2 = PathBuf::from("/models/music.safetensors");
        supervisor
            .load_model(
                Modality::Music,
                "music",
                &p2,
                500_000_000,
                1_000_000_000,
                Some(AcceleratorBackend::Cuda),
            )
            .unwrap();

        // Now trigger fallback on Music adapter
        supervisor
            .fallback_to_cpu(Modality::Music, "DSP accelerator transient fault")
            .unwrap();

        // Verify Music degraded to CPU
        let music_health = supervisor.health_check(Modality::Music).unwrap();
        assert_eq!(music_health.active_backend, AcceleratorBackend::Cpu);

        // Verify LLM remained healthy on CUDA without any disruption!
        let llm_health = supervisor.health_check(Modality::Llm).unwrap();
        assert_eq!(llm_health.active_backend, AcceleratorBackend::Cuda);
    }

    #[test]
    fn test_supervisor_hot_unload_and_health_check_broadcasting() {
        let hardware = create_test_hardware_profile(10_000_000_000, false, 0);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let event_bus = Arc::new(EventBus::new(32, 16));
        let mut sub = event_bus.subscribe();

        let mut supervisor = RuntimeSupervisor::new(hardware, privacy, Some(event_bus));

        let p = PathBuf::from("/models/imagegen.bin");
        supervisor
            .load_model(
                Modality::ImageGeneration,
                "sd-turbo",
                &p,
                1_000_000_000,
                0,
                Some(AcceleratorBackend::Cpu),
            )
            .unwrap();

        // Check event received
        let ev1 = sub.try_recv().unwrap();
        assert_eq!(ev1.event.event_type(), "adapter_health_changed");

        // Unload model
        supervisor.unload_model(Modality::ImageGeneration).unwrap();
        assert_eq!(
            supervisor.get_state(Modality::ImageGeneration),
            Some(&AdapterState::Unloaded)
        );

        let ev2 = sub.try_recv().unwrap();
        assert_eq!(ev2.event.event_type(), "adapter_health_changed");
    }

    #[test]
    fn test_supervisor_builds_complete_system_snapshot() {
        let hardware = create_test_hardware_profile(10_000_000_000, false, 0);
        let privacy = Arc::new(PrivacyPolicyEngine::default());
        let supervisor = RuntimeSupervisor::new(hardware, privacy, None);

        let snapshot = supervisor.build_snapshot(VerificationTier::ProvenOnHost);
        assert_eq!(snapshot.contract_version, "1.0.0");
        assert_eq!(snapshot.verification_tier, VerificationTier::ProvenOnHost);
        assert_eq!(snapshot.capabilities.len(), 7);
        assert_eq!(snapshot.adapter_health.len(), 7);
        assert!(snapshot.privacy.local_only_mode);
    }
}
