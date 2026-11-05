# Stage 9 Technical Execution Prompt: Hybrid Cloud GPU & Secure Distributed Compute

## 1. Objective & Dependencies
- **Objective:** Deliver consent-gated hybrid cloud GPU execution and distributed compute dispatch (`crates/miniai-cloud`) with cryptographic `CloudConsentToken` enforcement, mTLS transport, immutable consent auditing, and instant local CPU fallback.
- **Dependencies:** Stage 8 Gate Verified.
- **Active Skills:** `miniai-security-privacy`, `miniai-offline-first`, `miniai-hardware-optimization`, `miniai-containers-dev`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 9.1: Consent Governance & Immutable Audit Ledger**
  - *Subphase 9.1.1:* Scoped `CloudConsentToken` lifecycle (`Grant`, `Verify`, `Revoke`, `Expire`) persisted to an append-only SQLite audit table.
- **Phase 9.2: Encrypted Remote Compute Dispatcher & Automatic Local Fallback**
  - *Subphase 9.2.1:* mTLS remote worker client that verifies `PrivacyPolicyEngine::authorize_cloud_execution` before serializing any payload, and automatically falls back to local CPU/GPU if consent is revoked or the remote node fails.
- **Phase 9.3: Flutter Hybrid Compute & Consent Inspector UI**
  - *Subphase 9.3.1:* Explicit per-task consent dialog displaying target endpoint, payload byte size, modality scope, expiration countdown, and instant kill-switch (`LocalOnlyPrivacyMode`).

## 3. Security & Edge Cases
- Zero bytes may leave the device when `LocalOnlyPrivacyMode == true`, when `user_acknowledged == false`, or when `current_epoch_secs >= expires_at_epoch_secs`.
- Revoking consent mid-job must immediately terminate the remote request and fall back locally.

## 4. Acceptance Tests & Exit Criteria
- 100% of unauthorized, expired, or mismatched cloud requests fail closed in security tests.
- >= 90% coverage on `miniai-cloud` and consent policy enforcement.

