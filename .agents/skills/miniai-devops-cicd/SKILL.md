---
name: miniai-devops-cicd
description: >-
  Guides DevOps, GitHub Actions CI/CD workflows, release packaging, and artifact aggregation across
  Windows, Linux, macOS, Android, and iOS for MiniAI under .github/workflows/ and scripts/.
---

# MiniAI DevOps & CI/CD Pipeline Skill

Use this skill when configuring GitHub Actions workflows (`ci.yml`, `release.yml`), writing packaging scripts (`scripts/package_release.py`), or auditing release artifacts.

## CI/CD Pipeline Architecture
1. **Multi-Platform Matrix Testing (`.github/workflows/ci.yml`)**:
   - `governance-and-python-ml`: Runs `uv run pytest --cov=miniai_ml --cov-fail-under=90` on Ubuntu, Windows, and macOS.
   - `rust-core`: Runs `cargo fmt`, `cargo clippy -D warnings`, and `cargo test --workspace` on Ubuntu, Windows, and macOS.
   - `flutter-cross-platform`: Runs `flutter pub get`, `flutter analyze --no-pub`, and `flutter test --coverage` on macOS.
   - `integration-and-gates`: Runs end-to-end verification via `python scripts/verify_stage10.py`.
2. **Release Artifact Pipeline (`.github/workflows/release.yml`)**:
   - `build-and-test-rust`: Builds release binaries for `miniai-server` and `miniai-daemon` and uploads them via `actions/upload-artifact@v4`.
   - `build-flutter-cross-platform`: Compiles Android APK/AAB and iOS IPA artifacts.
   - `package-and-checksum-release`:
     - Downloads all compiled artifacts via `actions/download-artifact@v4`.
     - Executes `scripts/package_release.py --version <version>` to produce bundles, CycloneDX v1.5 SBOM, and SHA-256 manifests.
     - Verifies cryptographic digests with `--verify`.
     - Publishes to GitHub Releases for tag pushes.

## Verification
- Validate workflow YAML syntax: check action versions (`actions/checkout@v4`, `actions/upload-artifact@v4`, `actions/download-artifact@v4`).
- Run `python scripts/package_release.py --version 1.0.0 --verify` locally.

