# MiniAI — Security & Privacy Policy (`SECURITY.md`)

**Document ID:** `MINIAI-SEC-001`  
**Version:** `1.0.0`

---

## 1. Security & Privacy Architecture

MiniAI is architected from the ground up around **local-first data sovereignty** and **defense-in-depth**:

1. **Default `LocalOnlyPrivacyMode`**:
   - On startup, `PrivacyPolicyEngine` initializes with `local_only_mode = true`.
   - While active, any attempt to invoke remote cloud inference or distributed compute fails closed with `MiniAiError::PrivacyModeBlocked`.
2. **Explicit, Scoped, Time-Bounded Cloud Consent (`CloudConsentToken`)**:
   - Hybrid cloud compute requires:
     - `local_only_mode == false`
     - `user_acknowledged == true`
     - Exact match on `modality` and `target_endpoint`
     - `current_epoch_secs < expires_at_epoch_secs`
3. **Cryptographic Model & Dataset Integrity**:
   - Every model weight file, LoRA adapter, or dataset archive is verified against a 64-character lowercase hex SHA-256 digest before loading.
4. **Path Traversal Prevention**:
   - Relative paths supplied to the model or dataset registry are sanitized by `validate_safe_relative_path`, rejecting `..`, null bytes, Windows drive prefixes (`C:`), UNC paths (`\\`), and leading slashes (`/`).
5. **Memory & Process Isolation**:
   - Core Rust crates enforce `#![forbid(unsafe_code)]`.
   - Model runtimes execute through decoupled `ModalityAdapter` instances so a malformed tensor or adapter fault cannot compromise the core supervisor.

---

## 2. Secret & Supply-Chain Hygiene

- **Zero Secrets in Repository**: API tokens, credentials, private keys, and `.env` files are strictly excluded via `.gitignore` and scanned automatically by `scripts/verify_stage0.py` and `.github/workflows/security.yml`.
- **GitHub Actions Least Privilege**: Workflows default to `permissions: contents: read`.

---

## 3. Reporting a Vulnerability

If you discover a security or privacy vulnerability in MiniAI:
1. Do not open a public issue containing exploit payloads or active credentials.
2. Document the reproduction steps, affected component (`crates/`, `ui/`, or `ml/`), and impact classification.
3. Submit a private security advisory via the GitHub Security tab of the repository.

