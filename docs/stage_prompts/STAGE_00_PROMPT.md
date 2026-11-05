# Stage 0 Technical Execution Prompt: Repository, Governance, Architecture, Skills, Rules, CI & Foundation

## 1. Objective & Scope
Establish the complete production engineering foundation for **MiniAI**:
- Governance & architecture documents (`CLAUDE.md`, `GLOBAL_ENGINEERING_CONTRACT.md`, `AGENTS.md`, `REQUIREMENTS.md`, `ARCHITECTURE.md`, `ADR.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `CODEBASE_GUIDE.md`, `USER_GUIDE.md`, `DEVELOPMENT_SETUP.md`, `TESTING.md`, `SECURITY.md`, `CONTRIBUTING.md`, `RELEASE_CERTIFICATION.md`, `README.md`, append-only `TODO.md` and `CHANGELOG.md`).
- Antigravity rules (`.agents/rules/*.md`) and 18 reusable skills (`.agents/skills/*/SKILL.md` + `SKILLS_INDEX.md`).
- Working tri-language workspace foundation:
  - Rust Core (`crates/miniai-core`)
  - Flutter Cross-Platform Shell (`ui/miniai_app`)
  - Python ML & Governance Tooling (`ml/src/miniai_ml`)
- Containerized development environment (`Dockerfile`, `docker-compose.yml`, `.devcontainer/devcontainer.json`).
- Multi-platform GitHub Actions workflows (`.github/workflows/ci.yml`, `security.yml`, `release.yml`) across Windows, Linux, macOS, Android, and iOS.
- Remote GitHub repository integration and verification.

## 2. Phases, Subphases & Tasks
- **Phase 0.1:** Governance Documents, Rules & 18 Skills (`S0-P1-T1` – `S0-P1-T5`)
- **Phase 0.2:** Rust Core (`miniai-core`), Flutter UI (`miniai_app`), and Python ML (`miniai_ml`) Foundations (`S0-P2-T1` – `S0-P2-T6`)
- **Phase 0.3:** Containers, GitHub Actions, Remote Git Integration & Independent Audit (`S0-P3-T1` – `S0-P3-T4`)

## 3. API & Data Contracts
- `Modality`: `llm`, `vision`, `image_generation`, `speech_asr`, `speech_tts`, `music`, `video`.
- `AcceleratorBackend`: `cpu`, `cuda`, `vulkan`, `metal`, `direct_ml`, `rocm`, `core_ml`, `nnapi`.
- `PrivacyPolicyState`: `local_only_mode: bool`, `active_consent: Option<CloudConsentToken>`.
- `SystemSnapshot`: Complete serializable system status across Rust, Dart, and Python.

## 4. Security & Edge Cases
- Reject path traversal (`..`, drive letters, root slashes) in `validate_safe_relative_path`.
- Verify SHA-256 digests (`verify_sha256_digest`) in constant/deterministic hex comparison.
- Block remote execution when `local_only_mode == true` or `CloudConsentToken` is missing, unacknowledged, mismatched, or expired.

## 5. Acceptance Tests & Exit Criteria
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass (`0` errors, `>= 90%` coverage on `miniai-core`).
- `uv run pytest --cov=miniai_ml --cov-fail-under=90` passes (`>= 90%` coverage).
- `flutter analyze --no-pub` and `flutter test --coverage` pass (`100%` passing tests, `>= 90%` model/controller coverage).
- `uv run --project ml python scripts/verify_stage0.py` exits `0`.
- Independent audit (`independent-qa-auditor` & `arch-security-reviewer`) returns `PASS`.

