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



---

## 5. Stage 1–9 Certified Milestones Summary

- [x] **Stage 1 (Core Storage & Local IPC)**: Tag `v0.1.0`, commit `627b0be`. Transactional SQLite storage with WAL mode, authenticated IPC, live event bus, Flutter shell navigation rail/bar.
- [x] **Stage 2 (Pluggable Model Runtimes & Supervisor)**: Tag `v0.2.0`, commit `1aa5ba3`. Dynamic hardware probing, memory admission ceiling, decoupled modality runtime supervisor, automatic CPU fallback.
- [x] **Stage 3 (Local LLM Inference & Chat)**: Tag `v0.3.0`, commit `684dc31`. GGUF parser, greedy/temperature/top-p sampler, KV cache, streaming tokens, multi-branch conversation UI.
- [x] **Stage 4 (Local Fine-Tuning & Distillation Pipeline)**: Tag `v0.4.0`, commit `e161474`. `miniai_ml` PyTorch package, dataset pipeline, LoRA/QLoRA, distillation trainer, SHA-256 provenance manifest, Flutter Training Studio.
- [x] **Stage 5 (Offline Local RAG & Document Intelligence)**: Tag `v0.5.0`, commit `9666c5a`. `miniai-rag` vector store, chunking engine, exact cosine similarity, citation tracking, Flutter Knowledge Base.
- [x] **Stage 6 (Vision & Canvas Studio)**: Tag `v0.6.0`, commit `29e1eb7`. `miniai-vision` OCR/scene captioning/VQA, `miniai-imagegen` scheduled denoising diffusion/inpainting/PNG provenance, Flutter Canvas Studio.
- [x] **Stage 7 (Audio, Speech & Sound Studio)**: Tag `v0.7.0`, commit `e2f0a52`. `miniai-speech-asr` 16 kHz resampler/VAD, `miniai-speech-tts` formant synthesizer/voice loop, `miniai-music` 4-track polyphonic synthesizer, Flutter Sound Studio.
- [x] **Stage 8 (Video Understanding, Synthesis & Workflows)**: Tag `v0.8.0`, commit `d47ea9f`. `miniai-video` bounded frame buffer pool, scene/motion detection, temporal VQA, generative video synthesis, composable Kahn's algorithm multimodal DAG, Flutter Video Studio.
- [x] **Stage 9 (Hybrid Cloud GPU & Distributed Compute)**: Tag `v0.9.0`, commit `faf5298`. `miniai-cloud` scoped `CloudConsentToken` lifecycle, append-only SQLite ledger, TLS 1.3/mTLS dispatcher, automatic local CPU fallback on 500/timeout/revocation, Flutter Consent Inspector & emergency kill-switch.

---

## 6. Stage 10 Gate Certification Checklist (`v1.0.0`)

- [x] **System-Wide Stress & Property-Based Fuzzing (`tests/fuzz_and_stress_tests.rs`)**:
  - `Modality::Llm`: Parameter fuzzing, runaway token limits, UTF-8 boundaries, injection delimiters, and tokenizer decoding.
  - `Modality::Vision`: Zero-byte inputs, truncated headers, corrupt chunks, zero-dimension rejection, and `MAX_IMAGE_DIMENSION` bomb guards.
  - `Modality::ImageGen`: Out-of-bounds guidance scales, step counts, non-multiples of 8, and malformed PNG chunk/CRC error handling in `PngDecoder`.
  - `Modality::SpeechAsr`: Truncated WAV/FLAC/Ogg headers, corrupt payloads, resampler bounds, and VAD silence discrimination.
  - `Modality::SpeechTts`: Empty text, SSML/XML injection resilience, phonetic chunking, and 22.05 kHz acoustic synthesis.
  - `Modality::Music`: Tempo BPM and duration limits, harmonic scale validation, and polyphonic 4-track synthesis.
  - `Modality::Video`: Malformed container headers, 1 MB buffer pool memory exhaustion fail-closed enforcement, and Kahn's algorithm cycle rejection in `MediaWorkflowGraph`.
  - Multi-threaded concurrent execution stress across 6 parallel worker threads.
- [x] **Transactional SQLite Crash Resilience (`crates/miniai-storage/tests/recovery_tests.rs`)**:
  - Zero-byte file initialization and automatic schema migration.
  - Unclosed transaction rollback and atomic isolation.
  - Foreign key invariant preservation and orphan rejection.
  - Corrupt database page quarantine and backup recovery (`StorageEngine::quarantine_and_recover`).
  - Concurrent multi-threaded read/write stress under WAL mode.
- [x] **Zero-Vulnerability Security Audit**:
  - `cargo audit` scanning 89 dependency crates with 0 vulnerabilities detected.
  - Zero secret patterns, private keys, or credentials detected by repository governance audit.
- [x] **Reproducible Cross-Platform Packaging (`scripts/package_release.py`)**:
  - Release archives for Windows (`x86_64`), Linux (`x86_64`), macOS (`universal`), Android (`APK`/`AAB`), and iOS (`unsigned IPA`).
  - CycloneDX v1.5 Software Bill of Materials (SBOM) in JSON (`bom.json`) and XML (`bom.xml`) inventorying all 14 workspace crates, Python ML tooling, and Flutter UI components.
  - Cryptographic SHA-256 manifests (`.sha256`) and master `SHA256SUMS.txt`, verified 100% passing via `--verify`.
- [x] **Multi-OS CI/CD Release Automation (`.github/workflows/release.yml`)**:
  - Automated build matrix for Windows, Linux, macOS.
  - Automated Flutter Android/iOS checks and release artifact publishing.
- [x] **100% Passing Test Suites Across All Tiers**:
  - Rust workspace: 342/342 tests passing, 0 Clippy warnings, 0 fmt diffs.
  - Python ML: 39/39 tests passing with 97.28% statement coverage.
  - Flutter UI: 102/102 tests passing with 91.47% line coverage, 0 analyzer issues.
- [x] **Architectural Decision Record**:
  - `ADR-015` recorded and accepted in `ADR.md`.
- [x] **Master Verification Evidence**:
  - `docs/STAGE_10_VERIFICATION_REPORT.json` generated and verified.
