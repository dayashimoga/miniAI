# Rule 05: Security, Privacy & Local-First Enforcement

## Scope
Applies across all crates, UI packages, ML scripts, containers, and GitHub Actions workflows.

## Mandates
1. **Local-Only Privacy Mode (`LocalOnlyPrivacyMode`)**:
   - Enabled by default. When active, all outbound network requests for inference, telemetry, or cloud compute are blocked at the core policy layer.
2. **Explicit Cloud Consent**:
   - Hybrid cloud or distributed compute may only execute when `LocalOnlyPrivacyMode` is disabled AND a valid, non-expired `CloudConsentToken` explicitly authorized by the user is present.
3. **Artifact Integrity**:
   - All downloaded models, adapters, and datasets must be verified against expected SHA-256 digests before loading.
   - Reject path traversal (`..`, absolute drive overrides) in model registry and dataset paths.
4. **Zero Secret Exposure**:
   - Never log, commit, or echo tokens, API keys, credentials, or private keys.
   - Pin minimal permissions (`permissions: contents: read`) in GitHub Actions workflows unless elevated permissions are explicitly required for release artifacts.

