# Stage 10 Technical Execution Prompt: Cross-Platform Hardening, Certification, Packaging & Releases

## 1. Objective & Dependencies
- **Objective:** Perform full-system hardening, fuzzing, performance optimization, cross-platform packaging (Windows, Linux, macOS, Android, iOS), SBOM and SHA-256 release manifest generation, and final Release Certification (`v1.0.0`).
- **Dependencies:** Stage 9 Gate Verified.
- **Active Skills:** `miniai-release-certification`, `miniai-github-actions`, `miniai-security-privacy`, `miniai-evaluation-benchmarks`, `miniai-testing-qa`, `miniai-independent-audit`.

## 2. Phases & Subphases
- **Phase 10.1: System-Wide Stress, Fuzzing & Security Audit**
  - *Subphase 10.1.1:* Property-based and malformed-input fuzz tests across all 7 modality adapters, SQLite recovery tests, and zero-vulnerability dependency audit.
- **Phase 10.2: Cross-Platform Packaging & Checksum Automation**
  - *Subphase 10.2.1:* Reproducible release bundles for Windows (`x86_64`), Linux (`x86_64`), macOS (`arm64`/`x86_64`), Android (`APK`/`AAB`), and iOS (`IPA --no-codesign`), accompanied by CycloneDX SBOM and `.sha256` checksum manifests.
- **Phase 10.3: Final Independent Release Certification Gate**
  - *Subphase 10.3.1:* Full verification of `RELEASE_CERTIFICATION.md` and final sign-off by `independent-qa-auditor` and `arch-security-reviewer`.

## 3. Acceptance Tests & Exit Criteria
- 100% of unit, contract, integration, UI, offline, recovery, security, and performance tests pass across all workspaces.
- >= 90% measured test coverage across all critical business logic crates and modules.
- All release artifacts and `.sha256` digests verified in GitHub Actions release pipeline.

