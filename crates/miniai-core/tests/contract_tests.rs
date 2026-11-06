use miniai_core::{
    AcceleratorBackend, CloudConsentToken, HardwareDetector, Modality, PrivacyPolicyEngine,
    RuntimeRegistry, SystemSnapshot, VerificationTier,
};

#[test]
fn test_cross_language_json_contract_serialization() {
    let registry = RuntimeRegistry::with_default_cpu_adapters().expect("registry init");
    let hardware = HardwareDetector::detect_host_profile();
    let mut privacy = PrivacyPolicyEngine::new();
    privacy.set_local_only_mode(false);
    privacy
        .grant_cloud_consent(CloudConsentToken {
            token_id: "tok-contract-001".to_string(),
            modality: Modality::Llm,
            target_endpoint: "https://compute.miniai.example".to_string(),
            max_payload_bytes: 65536,
            issued_at_epoch_secs: 1_700_000_000,
            expires_at_epoch_secs: 1_700_003_600,
            user_acknowledged: true,
        })
        .expect("valid consent token");

    let snapshot = registry
        .build_snapshot(
            hardware,
            privacy.state().clone(),
            VerificationTier::ProvenOnHost,
        )
        .expect("build snapshot");

    let value = serde_json::to_value(&snapshot).expect("serialize to json value");
    assert_eq!(value["contract_version"], "1.0.0");
    assert_eq!(value["verification_tier"], "PROVEN_ON_HOST");
    assert_eq!(value["privacy"]["local_only_mode"], false);
    assert_eq!(
        value["privacy"]["active_consent"]["modality"],
        serde_json::Value::String("llm".to_string())
    );
    assert_eq!(value["capabilities"].as_array().unwrap().len(), 7);
    assert_eq!(
        value["hardware"]["detected_backends"][0],
        serde_json::Value::String("cpu".to_string())
    );

    let round_trip: SystemSnapshot =
        serde_json::from_value(value).expect("deserialize from json value");
    assert_eq!(round_trip.capabilities.len(), 7);
    assert!(round_trip
        .hardware
        .detected_backends
        .contains(&AcceleratorBackend::Cpu));
}
