use crate::error::CloudError;
use miniai_core::CloudConsentToken;
use sha2::{Digest, Sha256};

/// Maximum permissible validity duration for any single consent token (24 hours).
pub const MAX_CONSENT_DURATION_SECS: u64 = 86_400;

/// Default maximum permissible single payload ceiling (100 MB).
pub const MAX_PAYLOAD_CEILING_BYTES: u64 = 104_857_600;

/// Validator enforcing defense-in-depth criteria on `CloudConsentToken` instances.
pub struct TokenValidator;

impl TokenValidator {
    /// Validates all structural, security, and cryptographic boundary rules for a `CloudConsentToken`.
    pub fn validate(token: &CloudConsentToken, allow_loopback: bool) -> Result<(), CloudError> {
        let tid = token.token_id.trim();
        if tid.is_empty() {
            return Err(CloudError::InvalidToken {
                reason: "Token ID cannot be empty or blank".to_string(),
            });
        }

        if tid.len() > 128 {
            return Err(CloudError::InvalidToken {
                reason: "Token ID length exceeds 128 characters".to_string(),
            });
        }

        if !token.user_acknowledged {
            return Err(CloudError::InvalidToken {
                reason: "User acknowledgement (user_acknowledged=true) is mandatory".to_string(),
            });
        }

        // Endpoint validation
        let ep = token.target_endpoint.trim();
        if !ep.starts_with("https://") {
            return Err(CloudError::InsecureEndpoint {
                endpoint: token.target_endpoint.clone(),
            });
        }

        let host_part = ep.strip_prefix("https://").unwrap_or("");
        let path_or_query_start = host_part.find(['/', '?']).unwrap_or(host_part.len());
        let authority = &host_part[..path_or_query_start];

        let host = if authority.starts_with('[') {
            if let Some(end_bracket) = authority.find(']') {
                &authority[1..end_bracket]
            } else {
                authority
            }
        } else if let Some(colon_pos) = authority.rfind(':') {
            // Check if it's host:port (single colon) or unbracketed IPv6 (multiple colons)
            if authority.matches(':').count() == 1 {
                &authority[..colon_pos]
            } else {
                authority
            }
        } else {
            authority
        }
        .trim();

        if host.is_empty() {
            return Err(CloudError::InvalidToken {
                reason: "Target endpoint does not specify a valid host".to_string(),
            });
        }

        if !allow_loopback {
            let host_lower = host.to_lowercase();
            if host_lower == "localhost"
                || host_lower == "127.0.0.1"
                || host_lower == "0.0.0.0"
                || host_lower == "::1"
                || host_lower == "[::1]"
                || host_lower.starts_with("127.")
                || host_lower.ends_with(".localhost")
            {
                return Err(CloudError::InvalidToken {
                    reason: format!(
                        "Target endpoint host '{host}' targets local loopback; loopback endpoints are forbidden in production"
                    ),
                });
            }
        }

        // Time window validation
        if token.expires_at_epoch_secs <= token.issued_at_epoch_secs {
            return Err(CloudError::InvalidToken {
                reason: format!(
                    "Expiration epoch ({}s) must be strictly greater than issuance epoch ({}s)",
                    token.expires_at_epoch_secs, token.issued_at_epoch_secs
                ),
            });
        }

        let duration = token.expires_at_epoch_secs - token.issued_at_epoch_secs;
        if duration > MAX_CONSENT_DURATION_SECS {
            return Err(CloudError::InvalidToken {
                reason: format!(
                    "Consent token validity duration ({duration}s) exceeds maximum allowed limit ({MAX_CONSENT_DURATION_SECS}s / 24h)"
                ),
            });
        }

        // Payload ceiling validation
        if token.max_payload_bytes == 0 {
            return Err(CloudError::InvalidToken {
                reason: "max_payload_bytes must be strictly positive".to_string(),
            });
        }

        if token.max_payload_bytes > MAX_PAYLOAD_CEILING_BYTES {
            return Err(CloudError::InvalidToken {
                reason: format!(
                    "max_payload_bytes ({}) exceeds platform ceiling ({} bytes)",
                    token.max_payload_bytes, MAX_PAYLOAD_CEILING_BYTES
                ),
            });
        }

        Ok(())
    }

    /// Computes a deterministic SHA-256 fingerprint for a `CloudConsentToken`.
    pub fn compute_token_fingerprint(token: &CloudConsentToken) -> String {
        let canonical_str = format!(
            "id={}|modality={}|endpoint={}|max_bytes={}|issued={}|expires={}|ack={}",
            token.token_id.trim(),
            crate::types::modality_to_wire(token.modality),
            token.target_endpoint.trim(),
            token.max_payload_bytes,
            token.issued_at_epoch_secs,
            token.expires_at_epoch_secs,
            token.user_acknowledged
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verifies that a token matches its expected cryptographic fingerprint.
    pub fn verify_token_integrity(token: &CloudConsentToken, expected_fingerprint: &str) -> bool {
        let computed = Self::compute_token_fingerprint(token);
        computed.eq_ignore_ascii_case(expected_fingerprint.trim())
    }

    /// Generates an HMAC-SHA256 signature over the canonical token fingerprint using a secret key.
    pub fn sign_token(token: &CloudConsentToken, secret_key: &[u8]) -> String {
        let fingerprint = Self::compute_token_fingerprint(token);
        let mut hasher = Sha256::new();
        hasher.update(secret_key);
        hasher.update(b":consent_hmac:");
        hasher.update(fingerprint.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verifies that a token's HMAC-SHA256 signature matches the provided secret key.
    pub fn verify_token_signature(
        token: &CloudConsentToken,
        secret_key: &[u8],
        signature_hex: &str,
    ) -> bool {
        let expected = Self::sign_token(token, secret_key);
        expected.eq_ignore_ascii_case(signature_hex.trim())
    }
}
