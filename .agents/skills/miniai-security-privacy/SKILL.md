---
name: miniai-security-privacy
description: >-
  Guides security engineering, threat modeling, SHA-256 artifact integrity verification, path
  traversal protection, secret scanning, and explicit cloud consent validation across MiniAI.
  Activate on every stage gate and whenever handling files, downloads, IPC, or cloud compute.
---

# MiniAI Security & Privacy Engineering Skill

Use this skill when implementing security controls, artifact verification, consent gates, or running stage security audits.

## Mandatory Security Controls
1. **SHA-256 Integrity Verification**: Every external weight file, adapter, or dataset archive must match its expected 64-character hex SHA-256 digest before use.
2. **Path Sanitization**: Reject relative traversal (`..`), null bytes, or unauthorized symlink escapes when resolving model or dataset paths.
3. **Cryptographic Consent Tokens**: Cloud/distributed compute requires an explicit `CloudConsentToken` with `scope`, `target_endpoint`, `issued_at_epoch_secs`, and `expires_at_epoch_secs`.
4. **Secret Hygiene**: Never hardcode or log secrets, tokens, or private keys.

## Workflow Steps
1. Review [`references/threat_model_checklist.md`](./references/threat_model_checklist.md).
2. Enforce `verify_sha256_digest` and `validate_safe_relative_path` in Rust and Python.
3. Run secret and security checks via `scripts/verify_stage0.py`.

## Verification
- Ensure all security unit tests in `crates/miniai-core` and `ml/tests/` pass with 0 vulnerabilities.

