use miniai_cloud::{CloudError, ConsentAction, ConsentLedger, ConsentManager, ConsentStatus};
use miniai_core::{CloudConsentToken, Modality, PrivacyPolicyEngine};
use miniai_storage::StorageEngine;
use std::sync::Arc;

fn setup_test_manager() -> (ConsentManager, PrivacyPolicyEngine) {
    let storage = Arc::new(StorageEngine::in_memory().expect("in-memory db"));
    let ledger = Arc::new(ConsentLedger::new(storage));
    let manager = ConsentManager::new_with_loopback(ledger, true);
    let mut privacy = PrivacyPolicyEngine::new();
    privacy.set_local_only_mode(false); // Enable cloud for test
    (manager, privacy)
}

#[test]
fn test_grant_consent_fails_when_local_only_mode_active() {
    let (manager, mut privacy) = setup_test_manager();
    privacy.set_local_only_mode(true); // Active!

    let token = CloudConsentToken {
        token_id: "tok_blocked".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 2000,
        user_acknowledged: true,
    };

    match manager.grant_consent(token, 1050, &mut privacy) {
        Err(CloudError::PrivacyModeBlocked) => {}
        other => panic!("Expected PrivacyModeBlocked, got {:?}", other),
    }

    // Ledger should record rejection
    let rejected = manager
        .ledger()
        .query_by_action(ConsentAction::Rejected)
        .unwrap();
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].token_id, "tok_blocked");
}

#[test]
fn test_grant_consent_success_and_sync_with_privacy_engine() {
    let (manager, mut privacy) = setup_test_manager();

    let token = CloudConsentToken {
        token_id: "tok_granted_1".to_string(),
        modality: Modality::Vision,
        target_endpoint: "https://cloud.miniai.internal/v1/vision".to_string(),
        max_payload_bytes: 5 * 1024 * 1024,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };

    let managed = manager
        .grant_consent(token.clone(), 1050, &mut privacy)
        .unwrap();
    assert_eq!(managed.status, ConsentStatus::Active);
    assert_eq!(managed.cumulative_dispatched_bytes, 0);
    assert_eq!(managed.remaining_bytes(), 5 * 1024 * 1024);
    assert!(!managed.fingerprint.is_empty());

    // Verify sync with PrivacyPolicyEngine
    assert_eq!(
        privacy.state().active_consent.as_ref().unwrap().token_id,
        "tok_granted_1"
    );

    // Verify ledger
    let granted_events = manager
        .ledger()
        .query_by_action(ConsentAction::Granted)
        .unwrap();
    assert_eq!(granted_events.len(), 1);
    assert_eq!(granted_events[0].token_id, "tok_granted_1");
}

#[test]
fn test_verify_and_authorize_scenarios() {
    let (manager, mut privacy) = setup_test_manager();

    let token = CloudConsentToken {
        token_id: "tok_auth_test".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 10000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 2000,
        user_acknowledged: true,
    };
    manager.grant_consent(token, 1000, &mut privacy).unwrap();

    // 1. Success case
    assert!(manager
        .verify_and_authorize(
            "tok_auth_test",
            Modality::Llm,
            "https://cloud.miniai.internal/v1/infer",
            1000,
            1200,
            &privacy,
        )
        .is_ok());

    // 2. Local-only mode kill-switch blocks authorization
    privacy.set_local_only_mode(true);
    match manager.verify_and_authorize(
        "tok_auth_test",
        Modality::Llm,
        "https://cloud.miniai.internal/v1/infer",
        1000,
        1200,
        &privacy,
    ) {
        Err(CloudError::PrivacyModeBlocked) => {}
        other => panic!("Expected PrivacyModeBlocked, got {:?}", other),
    }
    privacy.set_local_only_mode(false);

    // 3. Token not found
    match manager.verify_and_authorize(
        "tok_nonexistent",
        Modality::Llm,
        "https://cloud.miniai.internal/v1/infer",
        1000,
        1200,
        &privacy,
    ) {
        Err(CloudError::TokenNotFound { .. }) => {}
        other => panic!("Expected TokenNotFound, got {:?}", other),
    }

    // 4. Modality mismatch
    match manager.verify_and_authorize(
        "tok_auth_test",
        Modality::Video, // Wrong!
        "https://cloud.miniai.internal/v1/infer",
        1000,
        1200,
        &privacy,
    ) {
        Err(CloudError::ScopeMismatch { .. }) => {}
        other => panic!("Expected ScopeMismatch, got {:?}", other),
    }

    // 5. Endpoint mismatch
    match manager.verify_and_authorize(
        "tok_auth_test",
        Modality::Llm,
        "https://evil.external.com/v1/infer", // Wrong!
        1000,
        1200,
        &privacy,
    ) {
        Err(CloudError::ScopeMismatch { .. }) => {}
        other => panic!("Expected ScopeMismatch, got {:?}", other),
    }

    // 6. Payload exceeds max allowed
    match manager.verify_and_authorize(
        "tok_auth_test",
        Modality::Llm,
        "https://cloud.miniai.internal/v1/infer",
        15000, // > 10000!
        1200,
        &privacy,
    ) {
        Err(CloudError::PayloadLimitExceeded { .. }) => {}
        other => panic!("Expected PayloadLimitExceeded, got {:?}", other),
    }

    // 7. Expired token
    match manager.verify_and_authorize(
        "tok_auth_test",
        Modality::Llm,
        "https://cloud.miniai.internal/v1/infer",
        1000,
        2500, // > expires_at (2000)
        &privacy,
    ) {
        Err(CloudError::ConsentExpired { .. }) => {}
        other => panic!("Expected ConsentExpired, got {:?}", other),
    }
}

#[test]
fn test_record_dispatch_and_quota_exhaustion() {
    let (manager, mut privacy) = setup_test_manager();

    let token = CloudConsentToken {
        token_id: "tok_quota".to_string(),
        modality: Modality::ImageGeneration,
        target_endpoint: "https://cloud.miniai.internal/v1/gen".to_string(),
        max_payload_bytes: 1000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 5000,
        user_acknowledged: true,
    };
    manager.grant_consent(token, 1000, &mut privacy).unwrap();

    // Dispatch 600 bytes
    manager.record_dispatch("tok_quota", 600, 1100).unwrap();
    let managed = manager.get_token("tok_quota").unwrap();
    assert_eq!(managed.cumulative_dispatched_bytes, 600);
    assert_eq!(managed.remaining_bytes(), 400);
    assert_eq!(managed.status, ConsentStatus::Active);

    // Attempting to dispatch 500 more exceeds remaining (400)
    match manager.verify_and_authorize(
        "tok_quota",
        Modality::ImageGeneration,
        "https://cloud.miniai.internal/v1/gen",
        500,
        1200,
        &privacy,
    ) {
        Err(CloudError::QuotaExceeded { .. }) => {}
        other => panic!("Expected QuotaExceeded, got {:?}", other),
    }

    // Dispatch remaining 400 bytes
    manager.record_dispatch("tok_quota", 400, 1300).unwrap();
    let managed_full = manager.get_token("tok_quota").unwrap();
    assert_eq!(managed_full.cumulative_dispatched_bytes, 1000);
    assert_eq!(managed_full.remaining_bytes(), 0);
    assert_eq!(managed_full.status, ConsentStatus::Exhausted);

    // Now any further verification fails as exhausted
    match manager.verify_and_authorize(
        "tok_quota",
        Modality::ImageGeneration,
        "https://cloud.miniai.internal/v1/gen",
        10,
        1400,
        &privacy,
    ) {
        Err(CloudError::QuotaExceeded { .. }) => {}
        other => panic!("Expected QuotaExceeded, got {:?}", other),
    }
}

#[test]
fn test_revocation_and_emergency_kill_switch() {
    let (manager, mut privacy) = setup_test_manager();

    let tok1 = CloudConsentToken {
        token_id: "tok_rev_1".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        max_payload_bytes: 5000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 3000,
        user_acknowledged: true,
    };
    let tok2 = CloudConsentToken {
        token_id: "tok_rev_2".to_string(),
        modality: Modality::Vision,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        max_payload_bytes: 5000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 3000,
        user_acknowledged: true,
    };

    manager.grant_consent(tok1, 1000, &mut privacy).unwrap();
    manager.grant_consent(tok2, 1000, &mut privacy).unwrap();

    assert_eq!(manager.list_active_tokens(1100).len(), 2);

    // Revoke token 1
    manager
        .revoke_consent(
            "tok_rev_1",
            "User toggled off cloud for LLM",
            1200,
            &mut privacy,
        )
        .unwrap();

    let t1 = manager.get_token("tok_rev_1").unwrap();
    assert_eq!(t1.status, ConsentStatus::Revoked);
    assert_eq!(t1.revoked_at_epoch_secs, Some(1200));

    // Verify token 1 fails verification
    match manager.verify_and_authorize(
        "tok_rev_1",
        Modality::Llm,
        "https://cloud.miniai.internal/v1",
        100,
        1250,
        &privacy,
    ) {
        Err(CloudError::ConsentRevoked { reason, .. }) => {
            assert!(reason.contains("User toggled off"));
        }
        other => panic!("Expected ConsentRevoked, got {:?}", other),
    }

    // Token 2 is still active
    assert_eq!(manager.list_active_tokens(1250).len(), 1);

    // Emergency kill switch revokes all
    let revoked_count = manager
        .revoke_all("Emergency Privacy Mode Activated", 1300, &mut privacy)
        .unwrap();
    assert_eq!(revoked_count, 1); // Only tok_rev_2 was active

    assert_eq!(manager.list_active_tokens(1350).len(), 0);

    // Complete audit history for tok_rev_1
    let history1 = manager.audit_history("tok_rev_1").unwrap();
    assert_eq!(history1.len(), 3); // Granted -> Revoked -> Rejected (when attempted)
}
