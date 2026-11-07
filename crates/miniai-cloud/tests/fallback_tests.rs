use miniai_cloud::{
    CloudConsentToken, CloudError, ComputeTaskRequest, ConsentAction, ConsentLedger,
    ConsentManager, DefaultLocalFallbackHandler, FallbackCoordinator, RemoteDispatcher,
    SimulatedRemoteTransport, TlsConfig,
};
use miniai_core::{Modality, PrivacyPolicyEngine};
use miniai_storage::StorageEngine;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

struct TestHarness {
    coordinator: FallbackCoordinator,
    manager: Arc<ConsentManager>,
    ledger: Arc<ConsentLedger>,
    privacy: PrivacyPolicyEngine,
}

fn setup_test_harness(should_fail_500: bool, should_timeout: bool) -> TestHarness {
    let storage = Arc::new(StorageEngine::in_memory().expect("in-memory db"));
    let ledger = Arc::new(ConsentLedger::new(storage));
    let manager = Arc::new(ConsentManager::new_with_loopback(ledger.clone(), true));
    let mut privacy = PrivacyPolicyEngine::new();
    privacy.set_local_only_mode(false); // Enable cloud for test

    let transport_impl = SimulatedRemoteTransport {
        should_fail_with_500: should_fail_500,
        should_timeout,
        ..Default::default()
    };
    let transport = Arc::new(transport_impl);

    let dispatcher = Arc::new(RemoteDispatcher::new(
        manager.clone(),
        TlsConfig::default(),
        transport,
    ));

    let local_handler = Arc::new(DefaultLocalFallbackHandler::default());
    let coordinator = FallbackCoordinator::new(dispatcher, local_handler, ledger.clone());

    TestHarness {
        coordinator,
        manager,
        ledger,
        privacy,
    }
}

#[test]
fn test_remote_success_does_not_trigger_fallback() {
    let mut harness = setup_test_harness(false, false);

    let token = CloudConsentToken {
        token_id: "tok_fb_ok".to_string(),
        modality: Modality::SpeechTts,
        target_endpoint: "https://cloud.miniai.internal/v1/tts".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    harness
        .manager
        .grant_consent(token, 1000, &mut harness.privacy)
        .unwrap();

    let req = ComputeTaskRequest {
        task_id: "task_ok".to_string(),
        token_id: "tok_fb_ok".to_string(),
        modality: Modality::SpeechTts,
        operation: "synthesize".to_string(),
        payload: b"tts_text".to_vec(),
        timeout_ms: 5000,
    };

    let resp = harness
        .coordinator
        .execute(
            "https://cloud.miniai.internal/v1/tts",
            req,
            1100,
            &harness.privacy,
            None,
        )
        .unwrap();

    assert!(resp.executed_remotely);
    assert_eq!(resp.status, "success");
    assert!(resp.fallback_reason.is_none());
    assert_eq!(resp.output_bytes, b"tts_text:processed_remotely");
}

#[test]
fn test_remote_500_failure_triggers_automatic_local_cpu_fallback() {
    let mut harness = setup_test_harness(true, false); // 500 error!

    let token = CloudConsentToken {
        token_id: "tok_fb_500".to_string(),
        modality: Modality::ImageGeneration,
        target_endpoint: "https://cloud.miniai.internal/v1/image".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    harness
        .manager
        .grant_consent(token, 1000, &mut harness.privacy)
        .unwrap();

    let req = ComputeTaskRequest {
        task_id: "task_500_fb".to_string(),
        token_id: "tok_fb_500".to_string(),
        modality: Modality::ImageGeneration,
        operation: "generate".to_string(),
        payload: b"image_prompt".to_vec(),
        timeout_ms: 5000,
    };

    let resp = harness
        .coordinator
        .execute(
            "https://cloud.miniai.internal/v1/image",
            req,
            1100,
            &harness.privacy,
            None,
        )
        .unwrap();

    // Fallback executed locally on CPU!
    assert!(!resp.executed_remotely);
    assert_eq!(resp.status, "success_fallback");
    assert_eq!(resp.worker_id, "local_cpu_adapter");
    assert_eq!(resp.output_bytes, b"image_prompt:processed_locally_cpu");
    assert!(resp.fallback_reason.unwrap().contains("500"));

    // Verify fallback audit event was recorded in ledger
    let rejections = harness
        .ledger
        .query_by_action(ConsentAction::Rejected)
        .unwrap();
    assert_eq!(rejections.len(), 1);
    assert_eq!(rejections[0].token_id, "tok_fb_500");
    assert!(rejections[0]
        .details_json
        .as_ref()
        .unwrap()
        .contains("fallback"));
}

#[test]
fn test_remote_timeout_triggers_automatic_local_fallback() {
    let mut harness = setup_test_harness(false, true); // Timeout!

    let token = CloudConsentToken {
        token_id: "tok_fb_to".to_string(),
        modality: Modality::SpeechAsr,
        target_endpoint: "https://cloud.miniai.internal/v1/asr".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    harness
        .manager
        .grant_consent(token, 1000, &mut harness.privacy)
        .unwrap();

    let req = ComputeTaskRequest {
        task_id: "task_to_fb".to_string(),
        token_id: "tok_fb_to".to_string(),
        modality: Modality::SpeechAsr,
        operation: "transcribe".to_string(),
        payload: b"audio_wav".to_vec(),
        timeout_ms: 3000,
    };

    let resp = harness
        .coordinator
        .execute(
            "https://cloud.miniai.internal/v1/asr",
            req,
            1100,
            &harness.privacy,
            None,
        )
        .unwrap();

    assert!(!resp.executed_remotely);
    assert_eq!(resp.status, "success_fallback");
    assert_eq!(resp.output_bytes, b"audio_wav:processed_locally_cpu");
    assert!(resp.fallback_reason.unwrap().contains("timed out"));
}

#[test]
fn test_local_only_privacy_mode_falls_back_locally() {
    let mut harness = setup_test_harness(false, false);

    let token = CloudConsentToken {
        token_id: "tok_privacy_fb".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    harness
        .manager
        .grant_consent(token, 1000, &mut harness.privacy)
        .unwrap();

    // User enables Local-Only Privacy Mode
    harness.privacy.set_local_only_mode(true);

    let req = ComputeTaskRequest {
        task_id: "task_priv_fb".to_string(),
        token_id: "tok_privacy_fb".to_string(),
        modality: Modality::Llm,
        operation: "generate".to_string(),
        payload: b"private_prompt".to_vec(),
        timeout_ms: 5000,
    };

    let resp = harness
        .coordinator
        .execute(
            "https://cloud.miniai.internal/v1",
            req,
            1100,
            &harness.privacy,
            None,
        )
        .unwrap();

    assert!(!resp.executed_remotely);
    assert_eq!(resp.status, "success_fallback");
    assert_eq!(resp.output_bytes, b"private_prompt:processed_locally_cpu");
    assert!(resp
        .fallback_reason
        .unwrap()
        .contains("Local-Only Privacy Mode"));
}

#[test]
fn test_cancellation_does_not_fall_back() {
    let harness = setup_test_harness(false, false);

    let req = ComputeTaskRequest {
        task_id: "task_cancelled".to_string(),
        token_id: "tok_cancel".to_string(),
        modality: Modality::Llm,
        operation: "generate".to_string(),
        payload: b"prompt".to_vec(),
        timeout_ms: 5000,
    };

    let cancel = AtomicBool::new(true);

    let result = harness.coordinator.execute(
        "https://cloud.miniai.internal/v1",
        req,
        1100,
        &harness.privacy,
        Some(&cancel),
    );

    // Must return OperationCancelled immediately without running fallback
    match result {
        Err(CloudError::OperationCancelled) => {}
        other => panic!("Expected OperationCancelled, got {:?}", other),
    }
}

#[test]
fn test_custom_fallback_handler_and_failure() {
    let storage = Arc::new(StorageEngine::in_memory().expect("in-memory db"));
    let ledger = Arc::new(ConsentLedger::new(storage));
    let manager = Arc::new(ConsentManager::new_with_loopback(ledger.clone(), true));
    let privacy = PrivacyPolicyEngine::new();

    let transport = Arc::new(SimulatedRemoteTransport {
        should_fail_with_500: true,
        ..Default::default()
    });

    let dispatcher = Arc::new(RemoteDispatcher::new(
        manager,
        TlsConfig::default(),
        transport,
    ));

    // Custom local handler output
    let custom_handler = Arc::new(DefaultLocalFallbackHandler {
        custom_output: Some(b"custom_cpu_output".to_vec()),
    });
    let coordinator = FallbackCoordinator::new(dispatcher.clone(), custom_handler, ledger.clone());

    let req = ComputeTaskRequest {
        task_id: "task_custom".to_string(),
        token_id: "tok_missing".to_string(),
        modality: Modality::Llm,
        operation: "generate".to_string(),
        payload: b"input".to_vec(),
        timeout_ms: 1000,
    };

    let resp = coordinator
        .execute(
            "https://cloud.miniai.internal/v1",
            req,
            1000,
            &privacy,
            None,
        )
        .unwrap();
    assert_eq!(resp.output_bytes, b"custom_cpu_output");

    // Failing local handler
    struct FailingHandler;
    impl miniai_cloud::LocalFallbackHandler for FailingHandler {
        fn execute_local(&self, _req: &ComputeTaskRequest) -> Result<Vec<u8>, CloudError> {
            Err(CloudError::RemoteTransportError {
                reason: "Simulated hardware fault".to_string(),
            })
        }
    }

    let failing_coordinator =
        FallbackCoordinator::new(dispatcher, Arc::new(FailingHandler), ledger);

    let req_fail = ComputeTaskRequest {
        task_id: "task_fail".to_string(),
        token_id: "tok_missing".to_string(),
        modality: Modality::Llm,
        operation: "generate".to_string(),
        payload: b"input".to_vec(),
        timeout_ms: 1000,
    };

    match failing_coordinator.execute(
        "https://cloud.miniai.internal/v1",
        req_fail,
        1000,
        &privacy,
        None,
    ) {
        Err(CloudError::FallbackFailed { reason }) => {
            assert!(reason.contains("Simulated hardware fault"));
        }
        other => panic!("Expected FallbackFailed, got {:?}", other),
    }
}
