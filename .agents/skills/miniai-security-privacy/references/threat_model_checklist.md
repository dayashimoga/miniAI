# MiniAI Threat Model & Mitigation Checklist

| Threat ID | Threat Description | Mitigation Control | Verification Test |
| :--- | :--- | :--- | :--- |
| `TM-01` | Silent data exfiltration to cloud endpoints | `PrivacyPolicyEngine` defaults to `local_only_mode = true` and blocks outbound requests | `test_local_only_blocks_cloud_execution` |
| `TM-02` | Tampered or malicious model weight files | Mandatory SHA-256 verification prior to adapter load | `test_sha256_integrity_verification` |
| `TM-03` | Arbitrary file read/write via path traversal in model IDs | `validate_safe_relative_path` rejects `..`, drive letters, and root slashes | `test_path_traversal_rejected` |
| `TM-04` | Expired or un-scoped cloud compute authorization | `CloudConsentToken` validates expiration timestamp and exact operation scope | `test_expired_consent_token_rejected` |
| `TM-05` | Secret leakage in repository or CI logs | Automated regex/entropy scanning in `verify_stage0.py` and CI `security.yml` | Stage Gate Security Check |

