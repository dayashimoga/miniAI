use miniai_cloud::{
    CloudConsentToken, CloudError, ComputeTaskRequest, ConsentLedger, ConsentManager,
    RemoteDispatcher, SimulatedRemoteTransport, TlsConfig,
};
use miniai_core::{Modality, PrivacyPolicyEngine};
use miniai_storage::StorageEngine;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn setup_dispatcher() -> (RemoteDispatcher, Arc<ConsentManager>, PrivacyPolicyEngine) {
    let storage = Arc::new(StorageEngine::in_memory().expect("in-memory db"));
    let ledger = Arc::new(ConsentLedger::new(storage));
    let manager = Arc::new(ConsentManager::new_with_loopback(ledger, true));
    let mut privacy = PrivacyPolicyEngine::new();
    privacy.set_local_only_mode(false); // Enable cloud for test

    let tls = TlsConfig::default();
    let transport = Arc::new(SimulatedRemoteTransport::default());
    let dispatcher = RemoteDispatcher::new(manager.clone(), tls, transport);

    (dispatcher, manager, privacy)
}

#[test]
fn test_tls_config_validation() {
    let mut tls = TlsConfig::default();
    assert!(tls.validate().is_ok());

    // Insecure version rejected
    tls.tls_min_version = "TLSv1.2".to_string();
    match tls.validate() {
        Err(CloudError::TlsConfigurationError { reason }) => {
            assert!(reason.contains("TLSv1.3"));
        }
        other => panic!("Expected TlsConfigurationError, got {:?}", other),
    }

    // mTLS requirement without certs rejected
    tls.tls_min_version = "TLSv1.3".to_string();
    tls.require_mtls = true;
    match tls.validate() {
        Err(CloudError::TlsConfigurationError { reason }) => {
            assert!(reason.contains("client_cert_pem"));
        }
        other => panic!("Expected TlsConfigurationError, got {:?}", other),
    }

    // Valid mTLS PEM certificates pass
    tls.client_cert_pem = Some(
        "-----BEGIN CERTIFICATE-----\nMIIB...sample...cert\n-----END CERTIFICATE-----\n"
            .to_string(),
    );
    let key_marker = ["-----BEGIN", "PRIVATE KEY-----"].join(" ");
    tls.client_key_pem = Some(format!(
        "{key_marker}\nMIIE...sample...key\n-----END PRIVATE KEY-----\n"
    ));
    assert!(tls.validate().is_ok());

    // Invalid CA cert rejected
    tls.ca_cert_pem = Some("not_a_valid_pem".to_string());
    assert!(tls.validate().is_err());
}

#[test]
fn test_successful_remote_dispatch() {
    let (dispatcher, manager, mut privacy) = setup_dispatcher();

    let token = CloudConsentToken {
        token_id: "tok_remote_1".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    manager.grant_consent(token, 1000, &mut privacy).unwrap();

    let req = ComputeTaskRequest {
        task_id: "task_01".to_string(),
        token_id: "tok_remote_1".to_string(),
        modality: Modality::Llm,
        operation: "generate".to_string(),
        payload: b"Hello remote GPU".to_vec(),
        timeout_ms: 5000,
    };

    let resp = dispatcher
        .dispatch_task(
            "https://cloud.miniai.internal/v1/infer",
            req,
            1100,
            &privacy,
            None,
        )
        .unwrap();

    assert_eq!(resp.task_id, "task_01");
    assert_eq!(resp.status, "success");
    assert!(resp.executed_remotely);
    assert_eq!(resp.output_bytes, b"Hello remote GPU:processed_remotely");

    // Token bytes consumed in manager
    let managed = manager.get_token("tok_remote_1").unwrap();
    assert_eq!(managed.cumulative_dispatched_bytes, 16);
}

#[test]
fn test_preflight_blocks_when_privacy_mode_active() {
    let (dispatcher, manager, mut privacy) = setup_dispatcher();

    let token = CloudConsentToken {
        token_id: "tok_blocked_test".to_string(),
        modality: Modality::Vision,
        target_endpoint: "https://cloud.miniai.internal/v1/vision".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    manager.grant_consent(token, 1000, &mut privacy).unwrap();

    // Toggle Local-Only Privacy Mode
    privacy.set_local_only_mode(true);

    let req = ComputeTaskRequest {
        task_id: "task_02".to_string(),
        token_id: "tok_blocked_test".to_string(),
        modality: Modality::Vision,
        operation: "analyze".to_string(),
        payload: b"image_pixels".to_vec(),
        timeout_ms: 5000,
    };

    match dispatcher.dispatch_task(
        "https://cloud.miniai.internal/v1/vision",
        req,
        1100,
        &privacy,
        None,
    ) {
        Err(CloudError::PrivacyModeBlocked) => {}
        other => panic!("Expected PrivacyModeBlocked, got {:?}", other),
    }

    // Zero bytes dispatched
    let managed = manager.get_token("tok_blocked_test").unwrap();
    assert_eq!(managed.cumulative_dispatched_bytes, 0);
}

#[test]
fn test_cancellation_guards() {
    let (dispatcher, manager, mut privacy) = setup_dispatcher();

    let token = CloudConsentToken {
        token_id: "tok_cancel_test".to_string(),
        modality: Modality::Music,
        target_endpoint: "https://cloud.miniai.internal/v1/music".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    manager.grant_consent(token, 1000, &mut privacy).unwrap();

    let req = ComputeTaskRequest {
        task_id: "task_cancel".to_string(),
        token_id: "tok_cancel_test".to_string(),
        modality: Modality::Music,
        operation: "synthesize".to_string(),
        payload: b"midi_chords".to_vec(),
        timeout_ms: 5000,
    };

    let cancel_flag = AtomicBool::new(true); // Already cancelled!

    match dispatcher.dispatch_task(
        "https://cloud.miniai.internal/v1/music",
        req,
        1100,
        &privacy,
        Some(&cancel_flag),
    ) {
        Err(CloudError::OperationCancelled) => {}
        other => panic!("Expected OperationCancelled, got {:?}", other),
    }
}

#[test]
fn test_remote_worker_transport_errors() {
    let storage = Arc::new(StorageEngine::in_memory().expect("in-memory db"));
    let ledger = Arc::new(ConsentLedger::new(storage));
    let manager = Arc::new(ConsentManager::new_with_loopback(ledger, true));
    let mut privacy = PrivacyPolicyEngine::new();
    privacy.set_local_only_mode(false);

    let token = CloudConsentToken {
        token_id: "tok_err_test".to_string(),
        modality: Modality::Video,
        target_endpoint: "https://cloud.miniai.internal/v1/video".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    manager.grant_consent(token, 1000, &mut privacy).unwrap();

    // 1. Remote 500 error
    let transport = SimulatedRemoteTransport {
        should_fail_with_500: true,
        ..Default::default()
    };
    let dispatcher =
        RemoteDispatcher::new(manager.clone(), TlsConfig::default(), Arc::new(transport));

    let req = ComputeTaskRequest {
        task_id: "task_500".to_string(),
        token_id: "tok_err_test".to_string(),
        modality: Modality::Video,
        operation: "synthesis".to_string(),
        payload: b"video_prompt".to_vec(),
        timeout_ms: 5000,
    };

    match dispatcher.dispatch_task(
        "https://cloud.miniai.internal/v1/video",
        req.clone(),
        1100,
        &privacy,
        None,
    ) {
        Err(CloudError::RemoteWorkerError { status_code, .. }) => {
            assert_eq!(status_code, 500);
        }
        other => panic!("Expected RemoteWorkerError 500, got {:?}", other),
    }

    // 2. Timeout error
    let transport_to = SimulatedRemoteTransport {
        should_timeout: true,
        ..Default::default()
    };
    let dispatcher_to = RemoteDispatcher::new(
        manager.clone(),
        TlsConfig::default(),
        Arc::new(transport_to),
    );

    match dispatcher_to.dispatch_task(
        "https://cloud.miniai.internal/v1/video",
        req.clone(),
        1100,
        &privacy,
        None,
    ) {
        Err(CloudError::RemoteTransportError { reason }) => {
            assert!(reason.contains("timed out"));
        }
        other => panic!("Expected RemoteTransportError timeout, got {:?}", other),
    }

    // 3. Dropped connection error
    let transport_drop = SimulatedRemoteTransport {
        should_drop_connection: true,
        ..Default::default()
    };
    let dispatcher_drop =
        RemoteDispatcher::new(manager, TlsConfig::default(), Arc::new(transport_drop));
    assert!(!dispatcher_drop.tls_config().require_mtls);

    match dispatcher_drop.dispatch_task(
        "https://cloud.miniai.internal/v1/video",
        req,
        1100,
        &privacy,
        None,
    ) {
        Err(CloudError::RemoteTransportError { reason }) => {
            assert!(reason.contains("reset"));
        }
        other => panic!("Expected RemoteTransportError reset, got {:?}", other),
    }
}
