use miniai_cloud::{
    CloudError, TokenValidator, MAX_CONSENT_DURATION_SECS, MAX_PAYLOAD_CEILING_BYTES,
};
use miniai_core::{CloudConsentToken, Modality};

#[test]
fn test_valid_token_passes_validation() {
    let token = CloudConsentToken {
        token_id: "tok_valid_12345".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud-gpu.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 50 * 1024 * 1024,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 1000 + 3600,
        user_acknowledged: true,
    };

    assert!(TokenValidator::validate(&token, false).is_ok());
}

#[test]
fn test_empty_or_oversized_token_id_rejected() {
    let mut token = CloudConsentToken {
        token_id: "   ".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 1000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 2000,
        user_acknowledged: true,
    };
    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("empty or blank"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }

    token.token_id = "a".repeat(129);
    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("exceeds 128"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }
}

#[test]
fn test_unacknowledged_token_rejected() {
    let token = CloudConsentToken {
        token_id: "tok_unacked".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 1000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 2000,
        user_acknowledged: false,
    };

    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("user_acknowledged=true"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }
}

#[test]
fn test_non_https_endpoint_rejected() {
    let token = CloudConsentToken {
        token_id: "tok_http".to_string(),
        modality: Modality::Llm,
        target_endpoint: "http://cloud.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 1000,
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 2000,
        user_acknowledged: true,
    };

    match TokenValidator::validate(&token, false) {
        Err(CloudError::InsecureEndpoint { endpoint }) => {
            assert!(endpoint.starts_with("http://"));
        }
        other => panic!("Expected InsecureEndpoint, got {:?}", other),
    }
}

#[test]
fn test_loopback_endpoints_rejected_in_production() {
    let loopbacks = [
        "https://localhost/v1",
        "https://127.0.0.1:8443/v1",
        "https://0.0.0.0/v1",
        "https://::1/v1",
        "https://[::1]:8443/v1",
        "https://127.1.2.3/v1",
    ];

    for ep in loopbacks {
        let token = CloudConsentToken {
            token_id: "tok_loopback".to_string(),
            modality: Modality::Vision,
            target_endpoint: ep.to_string(),
            max_payload_bytes: 1000,
            issued_at_epoch_secs: 1000,
            expires_at_epoch_secs: 2000,
            user_acknowledged: true,
        };

        match TokenValidator::validate(&token, false) {
            Err(CloudError::InvalidToken { reason }) => {
                assert!(
                    reason.contains("loopback"),
                    "Expected loopback rejection for {ep}"
                );
            }
            other => panic!("Expected loopback rejection for {ep}, got {:?}", other),
        }

        // Must succeed when allow_loopback is true (for local test harnesses)
        assert!(TokenValidator::validate(&token, true).is_ok());
    }
}

#[test]
fn test_time_window_validation() {
    let mut token = CloudConsentToken {
        token_id: "tok_time".to_string(),
        modality: Modality::Video,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        max_payload_bytes: 1000,
        issued_at_epoch_secs: 2000,
        expires_at_epoch_secs: 1500, // Inverted!
        user_acknowledged: true,
    };

    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("strictly greater than"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }

    // Equal epoch also rejected
    token.expires_at_epoch_secs = 2000;
    assert!(TokenValidator::validate(&token, false).is_err());

    // Exceeding 24 hours rejected
    token.expires_at_epoch_secs = token.issued_at_epoch_secs + MAX_CONSENT_DURATION_SECS + 1;
    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("exceeds maximum allowed limit"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }
}

#[test]
fn test_payload_limits_validation() {
    let mut token = CloudConsentToken {
        token_id: "tok_payload".to_string(),
        modality: Modality::ImageGeneration,
        target_endpoint: "https://cloud.miniai.internal/v1".to_string(),
        max_payload_bytes: 0, // Zero forbidden!
        issued_at_epoch_secs: 1000,
        expires_at_epoch_secs: 2000,
        user_acknowledged: true,
    };

    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("strictly positive"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }

    token.max_payload_bytes = MAX_PAYLOAD_CEILING_BYTES + 1;
    match TokenValidator::validate(&token, false) {
        Err(CloudError::InvalidToken { reason }) => {
            assert!(reason.contains("exceeds platform ceiling"));
        }
        other => panic!("Expected InvalidToken, got {:?}", other),
    }
}

#[test]
fn test_token_fingerprint_and_integrity_verification() {
    let token = CloudConsentToken {
        token_id: "tok_fp_1".to_string(),
        modality: Modality::SpeechAsr,
        target_endpoint: "https://cloud.miniai.internal/v1/asr".to_string(),
        max_payload_bytes: 1048576,
        issued_at_epoch_secs: 5000,
        expires_at_epoch_secs: 10000,
        user_acknowledged: true,
    };

    let fp1 = TokenValidator::compute_token_fingerprint(&token);
    assert_eq!(fp1.len(), 64);

    // Determinism
    let fp2 = TokenValidator::compute_token_fingerprint(&token);
    assert_eq!(fp1, fp2);
    assert!(TokenValidator::verify_token_integrity(&token, &fp1));

    // Tampered token changes fingerprint
    let mut tampered = token.clone();
    tampered.max_payload_bytes = 2048576;
    let fp_tampered = TokenValidator::compute_token_fingerprint(&tampered);
    assert_ne!(fp1, fp_tampered);
    assert!(!TokenValidator::verify_token_integrity(&tampered, &fp1));
}

#[test]
fn test_token_hmac_signature_and_tamper_rejection() {
    let token = CloudConsentToken {
        token_id: "tok_hmac_1".to_string(),
        modality: Modality::Llm,
        target_endpoint: "https://cloud-gpu.miniai.internal/v1/infer".to_string(),
        max_payload_bytes: 1048576,
        issued_at_epoch_secs: 5000,
        expires_at_epoch_secs: 10000,
        user_acknowledged: true,
    };

    let secret = b"my_super_secret_hmac_key_12345678";
    let wrong_secret = b"wrong_secret_key_87654321";

    let sig = TokenValidator::sign_token(&token, secret);
    assert_eq!(sig.len(), 64);

    // 1. Valid signature passes
    assert!(TokenValidator::verify_token_signature(&token, secret, &sig));

    // 2. Wrong secret key fails
    assert!(!TokenValidator::verify_token_signature(
        &token,
        wrong_secret,
        &sig
    ));

    // 3. Tampered payload fails
    let mut tampered = token.clone();
    tampered.max_payload_bytes = 9999999;
    assert!(!TokenValidator::verify_token_signature(
        &tampered, secret, &sig
    ));
}
