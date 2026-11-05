---
name: miniai-testing-qa
description: >-
  Guides multi-tier testing (unit, contract, integration, UI, offline, failure/recovery, security,
  and performance) and >=90% coverage enforcement across MiniAI. Activate during test authoring
  and stage gate validation across all stages.
---

# MiniAI Testing & Quality Assurance Skill

Use this skill to design, execute, and verify tests across Rust, Flutter, and Python.

## Mandatory Testing Tiers
1. **Unit Tests**: Every domain function, error branch, and state transition.
2. **Contract Tests**: Cross-language JSON schema compatibility (`Rust` <-> `Dart` <-> `Python`).
3. **Integration Tests**: Multi-module workflows.
4. **UI Widget Tests**: Accessibility semantics, privacy mode controls, and modality cards.
5. **Offline & Recovery Tests**: Disconnected operation, SHA-256 mismatch detection, and CPU fallback.
6. **Security Tests**: Path traversal blocking, consent token expiration, and secret scanning.

## Workflow Steps
1. Read [`references/coverage_commands.md`](./references/coverage_commands.md).
2. Run Rust tests: `cargo test --workspace`
3. Run Python tests with coverage: `uv run pytest --cov=miniai_ml --cov-fail-under=90`
4. Run Flutter analysis and widget tests: `flutter analyze --no-pub` and `flutter test --coverage`
5. Run master gate verifier: `uv run python scripts/verify_stage0.py`

## Verification
- Critical logic coverage must be >= 90% and required tests must pass at 100%.

