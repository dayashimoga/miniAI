# MiniAI — Release & Stage Gate Certification Specification (`RELEASE_CERTIFICATION.md`)

**Document ID:** `MINIAI-CERT-001`  
**Version:** `1.0.0`

---

## 1. Certification Authority & Gate Standard

No stage gate (`Stage 0` through `Stage 10`) or public release (`v0.x` / `v1.0.0`) may be marked certified until:
1. All code is **implemented, integrated, executed, tested, documented, and independently verified**.
2. Critical business logic achieves **>= 90% measured test coverage**.
3. Required release and gate test suites achieve a **100% pass rate**.
4. Zero fake/mock data, hardcoded demonstrations, placeholder buttons, or secret leaks exist.
5. Both `independent-qa-auditor` and `arch-security-reviewer` issue a `PASS` verdict.

---

## 2. Verification Classification Taxonomy

Every certified capability and test result is explicitly classified as:
- **`PROVEN_ON_HOST`**: Executed and verified directly on the host machine (`Windows 11 x86_64`, `rustc 1.99.1`, `Flutter 3.38.4`, `uv 0.9.5 / Python 3.14`).
- **`VERIFIED_IN_CI`**: Configured and verified via GitHub Actions runners (`windows-latest`, `ubuntu-latest`, `macos-latest`).
- **`HARDWARE_DEPENDENT`**: Accelerator-specific execution paths (`CUDA`, `Metal`, `Vulkan`, `DirectML`, `ROCm`, `CoreML`, `NNAPI`) where host lacks that specific physical hardware; deterministic CPU fallback (`AcceleratorBackend::Cpu`) is `PROVEN_ON_HOST`.

---

## 3. Cross-Platform Target Certification Matrix

| Target Platform | CI Runner | Build Command | Verification Classification |
| :--- | :--- | :--- | :--- |
| **Windows (`x86_64`)** | `windows-latest` + Local Host | `cargo test`, `flutter test`, `flutter build windows` | `PROVEN_ON_HOST` & `VERIFIED_IN_CI` |
| **Linux (`x86_64`)** | `ubuntu-latest` | `cargo test`, `flutter test`, `flutter build linux` | `VERIFIED_IN_CI` |
| **macOS (`arm64` / `x86_64`)** | `macos-latest` | `cargo test`, `flutter test`, `flutter build macos` | `VERIFIED_IN_CI` |
| **Android (`arm64-v8a`)** | `ubuntu-latest` (Android SDK) | `flutter build apk --release` | `VERIFIED_IN_CI` |
| **iOS (`arm64`)** | `macos-latest` (Xcode) | `flutter build ios --release --no-codesign` | `VERIFIED_IN_CI` |

---

## 4. Stage 0 Gate Certification Checklist

- [x] **Governance Suite**: `CLAUDE.md`, `GLOBAL_ENGINEERING_CONTRACT.md`, `AGENTS.md`, `REQUIREMENTS.md`, `ARCHITECTURE.md`, `IMPLEMENTATION_PLAN.md`, `CODEBASE_GUIDE.md`, `USER_GUIDE.md`, `DEVELOPMENT_SETUP.md`, `TESTING.md`, `SECURITY.md`, `CONTRIBUTING.md`, `ADR.md`, `ROADMAP.md`, `RELEASE_CERTIFICATION.md`, `README.md`, `TODO.md`, `CHANGELOG.md`.
- [x] **Antigravity Customizations**: 6 workspace rules in `.agents/rules/*.md` and 18 reusable skills in `.agents/skills/*/SKILL.md` + `SKILLS_INDEX.md`.
- [x] **Detailed Stage Prompts**: `docs/stage_prompts/STAGE_00_PROMPT.md` through `STAGE_10_PROMPT.md`.
- [x] **Tri-Language Foundation**:
  - Rust `crates/miniai-core` passing `fmt`, `clippy`, and `test` (`>= 90%` coverage).
  - Flutter `ui/miniai_app` passing `flutter analyze` and `flutter test --coverage` (`>= 90%` coverage on core models/controllers).
  - Python `ml/src/miniai_ml` passing `uv run pytest --cov=miniai_ml --cov-fail-under=90`.
- [x] **CI/CD & Containers**: `.github/workflows/ci.yml`, `security.yml`, `release.yml`, `Dockerfile`, `docker-compose.yml`, `.devcontainer/devcontainer.json`.

