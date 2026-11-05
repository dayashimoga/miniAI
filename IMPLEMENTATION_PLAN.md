# MiniAI — Staged Technical Implementation Plan (`IMPLEMENTATION_PLAN.md`)

**Document ID:** `MINIAI-PLAN-001`  
**Hierarchy Standard:** `Stage -> Phase -> Subphase -> Tasks -> Acceptance Tests`  
**Version:** `1.0.0`

---

## 1. Execution Governance

Every Stage delivers one complete, integrated, and independently usable vertical slice of MiniAI. Every Phase and Subphase specifies:
- **Objective & Dependencies**
- **Design & Implementation**
- **API / Data Contracts**
- **UI & Accessibility**
- **Security & Edge Cases**
- **Verification Tests & Measurable Exit Criteria**

Detailed executable technical prompts for each stage are maintained in [`docs/stage_prompts/`](./docs/stage_prompts/).

---

## Stage 0: Repository, Governance, Architecture, Skills, Rules, CI & Development Foundation

- **Objective:** Establish the complete engineering governance framework, Antigravity rules and 18 skills, GitHub repository & multi-OS CI workflows, containerized dev configuration, staged prompts, and verified Rust (`miniai-core`), Flutter (`miniai_app`), and Python (`miniai_ml`) foundation workspaces.
- **Dependencies:** None (Bootstrap Stage).
- **Active Skills:** `miniai-architecture`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-python-ml`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-containers-dev`, `miniai-github-actions`, `miniai-testing-qa`, `miniai-release-certification`, `miniai-documentation-logs`, `miniai-staged-implementation`, `miniai-independent-audit`.

### Phase 0.1: Repository Governance & Antigravity Customizations
- **Subphase 0.1.1: Core Contracts & Governance Documentation**
  - *Objective:* Create `CLAUDE.md`, `GLOBAL_ENGINEERING_CONTRACT.md`, `AGENTS.md`, `REQUIREMENTS.md`, `ARCHITECTURE.md`, `ADR.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `CODEBASE_GUIDE.md`, `USER_GUIDE.md`, `DEVELOPMENT_SETUP.md`, `TESTING.md`, `SECURITY.md`, `CONTRIBUTING.md`, `RELEASE_CERTIFICATION.md`, `README.md`, and strictly append-only `TODO.md` and `CHANGELOG.md`.
  - *Tasks:*
    - `S0-P1-T1`: Preserve supplied `CLAUDE.md` and integrate MiniAI engineering rules.
    - `S0-P1-T2`: Author all 19 mandatory governance and engineering documentation files.
    - `S0-P1-T3`: Author detailed stage execution prompts `docs/stage_prompts/STAGE_00_PROMPT.md` through `STAGE_10_PROMPT.md`.
  - *Acceptance Tests:*
    - `AT-S0-01`: All required governance files exist, are non-empty, and pass structural validation in `verify_stage0.py`.
    - `AT-S0-02`: `TODO.md` and `CHANGELOG.md` pass append-only chronological entry verification.
- **Subphase 0.1.2: Antigravity Rules & 18 Reusable Skills**
  - *Objective:* Author `.agents/rules/*.md` and `.agents/skills/*/SKILL.md` with YAML frontmatter, references, examples, and `SKILLS_INDEX.md`.
  - *Tasks:*
    - `S0-P1-T4`: Create 6 modular workspace rules in `.agents/rules/`.
    - `S0-P1-T5`: Create 18 reusable skills in `.agents/skills/` with supporting `references/` and `examples/`.
  - *Acceptance Tests:*
    - `AT-S0-03`: Every `SKILL.md` contains valid YAML frontmatter (`name`, `description`) and resolves all linked reference files.

### Phase 0.2: Tri-Language Workspace Foundation (Rust, Flutter, Python)
- **Subphase 0.2.1: Rust Core Foundation (`crates/miniai-core`)**
  - *Objective:* Implement core domain contracts (`Modality`, `AcceleratorBackend`, `HardwareProfile`, `PrivacyPolicyEngine`, `CloudConsentToken`, `RuntimeRegistry`, `ModalityAdapter`, SHA-256 & path sanitization utilities).
  - *API / Data Contracts:* Canonical `snake_case` JSON serialization (`SystemSnapshot`, `RuntimeCapability`, `PrivacyPolicyState`, `MiniAiError`).
  - *Security & Edge Cases:* Forbid `unsafe_code`; block cloud execution when `local_only_mode == true` or consent token is expired/mismatched; reject `..` path traversal and invalid SHA-256 digests.
  - *Tasks:*
    - `S0-P2-T1`: Create workspace `Cargo.toml` and `crates/miniai-core` library.
    - `S0-P2-T2`: Write comprehensive unit and contract tests covering all public types and error branches.
  - *Acceptance Tests:*
    - `AT-S0-04`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass with 0 warnings/errors and >=90% line coverage.
- **Subphase 0.2.2: Flutter Cross-Platform Shell Foundation (`ui/miniai_app`)**
  - *Objective:* Implement real cross-platform Flutter application displaying live system architecture snapshot, interactive `LocalOnlyPrivacyMode` toggle with consent validation, hardware backend selector (CPU-first), and registered modality adapter status cards.
  - *UI & Accessibility:* Responsive desktop/mobile layout, explicit `Semantics` labels, zero unconnected buttons or mock data.
  - *Tasks:*
    - `S0-P2-T3`: Create `ui/miniai_app` package with typed contract models, state controller, and adaptive dashboard screen.
    - `S0-P2-T4`: Implement widget, accessibility semantics, and state unit tests in `ui/miniai_app/test/`.
  - *Acceptance Tests:*
    - `AT-S0-05`: `flutter analyze --no-pub` reports zero issues and `flutter test --coverage` passes 100% of tests with >=90% coverage on models/controllers.
- **Subphase 0.2.3: Python ML & Governance Verification Package (`ml/`)**
  - *Objective:* Implement `uv`-managed `miniai_ml` package providing cross-language contract validation, deterministic seed management, SHA-256 dataset/checkpoint manifest verification, hardware environment probing (CPU-first), and repository governance auditing.
  - *Tasks:*
    - `S0-P2-T5`: Create `ml/pyproject.toml`, `ml/src/miniai_ml/`, and `scripts/verify_stage0.py`.
    - `S0-P2-T6`: Write `pytest` test suite in `ml/tests/` verifying contracts, security checks, and governance rules.
  - *Acceptance Tests:*
    - `AT-S0-06`: `uv run pytest --cov=miniai_ml --cov-fail-under=90` passes with >=90% measured coverage.

### Phase 0.3: Containers, GitHub Repository Integration & Multi-OS CI/CD
- **Subphase 0.3.1: Containerized Development Environment**
  - *Objective:* Provide reproducible `Dockerfile`, `docker-compose.yml`, and `.devcontainer/devcontainer.json` with non-root user execution.
  - *Tasks:*
    - `S0-P3-T1`: Author `Dockerfile`, `docker-compose.yml`, and `.devcontainer/devcontainer.json`.
  - *Acceptance Tests:*
    - `AT-S0-07`: Container files pass structural and security policy checks in `verify_stage0.py`.
- **Subphase 0.3.2: GitHub Repository & Multi-Platform GitHub Actions**
  - *Objective:* Create and verify `.github/workflows/ci.yml`, `.github/workflows/security.yml`, and `.github/workflows/release.yml` covering Windows, Linux, macOS, Android, and iOS on appropriate OS runners; integrate remote GitHub repository and verify push/branch protection status.
  - *Tasks:*
    - `S0-P3-T2`: Author multi-OS CI, security, and release workflows.
    - `S0-P3-T3`: Integrate remote GitHub repository (`dayashimoga/miniAI`), push `main` and `develop` branches, configure branch protection where supported by plan/API, and verify remote state.
    - `S0-P3-T4`: Execute independent QA and architecture/security subagent audits and record Stage 0 gate report.
  - *Acceptance Tests:*
    - `AT-S0-08`: `uv run python scripts/verify_stage0.py` exits `0`, remote repository state is verified via `git ls-remote`, and `independent-qa-auditor` returns `PASS`.

---

## Stage 1: Rust Core, Flutter Shell, Storage, APIs & Event System

- **Objective:** Deliver transactional SQLite storage (`crates/miniai-storage`), local HTTP/IPC server (`crates/miniai-server`), typed pub/sub event bus, and full Flutter shell integration with persistent workspace profiles and diagnostic logs.
- **Dependencies:** Stage 0 Gate Verified.
- **Active Skills:** `miniai-architecture`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-testing-qa`, `miniai-documentation-logs`, `miniai-staged-implementation`, `miniai-independent-audit`.
- **Phases & Subphases:**
  - **Phase 1.1: Transactional Storage & Schema Migrations (`crates/miniai-storage`)**
    - *Subphase 1.1.1:* SQLite WAL connection pool, deterministic migrations, settings/workspace/audit tables, and corruption detection.
    - *Subphase 1.1.2:* Repository CRUD contracts and crash-recovery integration tests.
  - **Phase 1.2: Typed Event Bus & Local IPC API (`crates/miniai-server`)**
    - *Subphase 1.2.1:* Async broadcast event bus (`SystemEvent` stream) with backpressure bounds.
    - *Subphase 1.2.2:* Localhost loopback API / FFI bridge with per-session authentication nonce.
  - **Phase 1.3: Flutter Shell Navigation, Settings & Event Stream Inspector**
    - *Subphase 1.3.1:* Persistent navigation rail/drawer, live event drawer, and SQLite-backed settings persistence.
    - *Subphase 1.3.2:* End-to-end Rust-to-Flutter contract and widget tests.
- **Exit Criteria:** >=90% coverage on `miniai-storage` and `miniai-server`, 100% passing Rust & Flutter tests, offline persistence verified across restarts.

---

## Stage 2: Model Registry, Downloads, Integrity, Hardware Detection & Runtime Adapters

- **Objective:** Deliver the local model registry, resumable HTTP/file downloader with mandatory SHA-256 verification, dynamic hardware capability detector, and pluggable runtime supervisor (`crates/miniai-registry`, `crates/miniai-runtime`).
- **Dependencies:** Stage 1 Gate Verified.
- **Active Skills:** `miniai-rust-core`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-flutter-ui`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 2.1: Model Manifest Catalog & Resumable Verified Downloads**
    - *Subphase 2.1.1:* Local model catalog schema, HTTP range-request resumable downloader, and atomic file promotion only after SHA-256 match.
    - *Subphase 2.1.2:* Offline local model import with quarantine and digest check.
  - **Phase 2.2: Hardware Probing & Admission Control**
    - *Subphase 2.2.1:* CPU instruction set (AVX2/AVX-512/NEON), RAM, and GPU/NPU probing with 85% memory admission guard.
  - **Phase 2.3: Pluggable Modality Runtime Supervisor & Registry UI**
    - *Subphase 2.3.1:* Process-isolated adapter lifecycle manager (`Load`, `HealthCheck`, `Unload`, `FallbackToCpu`).
    - *Subphase 2.3.2:* Flutter Model Manager & Hardware Inspector views with real download progress, checksum badges, and cancellation controls.
- **Exit Criteria:** Tampered weights deterministically rejected; interrupted downloads resume cleanly; CPU fallback proven when accelerator is absent.

---

## Stage 3: Offline LLM Inference, Chat, Streaming, Model Switching & Benchmarking

- **Objective:** Deliver complete offline LLM inference (`Modality::Llm`) with CPU-first GGUF/Candle/ONNX execution, token streaming, multi-session chat persistence, hot model switching, and quantitative LLM benchmarking.
- **Dependencies:** Stage 2 Gate Verified.
- **Active Skills:** `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-evaluation-benchmarks`, `miniai-flutter-ui`, `miniai-offline-first`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 3.1: Local LLM Runtime Adapter & Token Streamer**
    - *Subphase 3.1.1:* Deterministic CPU/GPU LLM inference engine with KV-cache management, stop-token handling, and cancellation tokens.
    - *Subphase 3.1.2:* Hot model switching with clean memory deallocation verified before loading next model.
  - **Phase 3.2: Multi-Turn Chat Persistence & Streaming UI**
    - *Subphase 3.2.1:* SQLite conversation tree storage (branching, regeneration, system prompts, sampling parameters).
    - *Subphase 3.2.2:* Flutter Chat Workspace with live markdown/code rendering and token speed telemetry.
  - **Phase 3.3: LLM Benchmarking Harness**
    - *Subphase 3.3.1:* Cold start, TTFT (Time-To-First-Token), tokens/sec, and peak RSS measurement exported to structured JSON and rendered in Flutter Benchmark View.
- **Exit Criteria:** 100% offline streaming chat E2E test passes on CPU; zero memory leak across 10 consecutive model switches; benchmark suite produces real host metrics.

---

## Stage 4: Dataset Management, Fine-Tuning, Training, Distillation & Evaluation

- **Objective:** Deliver local dataset curation, validation, LoRA/QLoRA fine-tuning, teacher-student knowledge distillation, checkpoint provenance tracking, and post-training evaluation (`ml/` + Rust supervisor + Flutter Training Studio).
- **Dependencies:** Stage 3 Gate Verified.
- **Active Skills:** `miniai-python-ml`, `miniai-training-distillation`, `miniai-evaluation-benchmarks`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 4.1: Dataset Curation, Validation & Splitting**
    - *Subphase 4.1.1:* JSONL/Parquet/CSV schema validator, deduplicator, deterministic `train`/`val`/`test` splitter, and SHA-256 dataset manifest generator.
  - **Phase 4.2: LoRA/QLoRA Fine-Tuning & Knowledge Distillation Engine**
    - *Subphase 4.2.1:* PyTorch CPU/GPU LoRA trainer and teacher-logit distillation pipeline with pause/resume checkpointing.
    - *Subphase 4.2.2:* `checkpoint_provenance.json` cryptographic lineage verification and adapter registration into Stage 2 Model Registry.
  - **Phase 4.3: Training Studio UI & Evaluation Comparison**
    - *Subphase 4.3.1:* Flutter dataset inspector, hyperparameter configurator, live loss curve monitor, and base-vs-tuned evaluation table.
- **Exit Criteria:** Micro fine-tuning and distillation jobs converge deterministically on CPU test fixtures; provenance manifests verified; >=90% coverage on `miniai_ml.training`.

---

## Stage 5: RAG, Document/Code Indexing & Secure Tool Integration

- **Objective:** Deliver local Retrieval-Augmented Generation (RAG), syntax-aware code/document chunking, local vector/BM25 hybrid search, citation grounding, and sandboxed local tool execution.
- **Dependencies:** Stage 4 Gate Verified.
- **Active Skills:** `miniai-rust-core`, `miniai-python-ml`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-flutter-ui`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 5.1: Document & Code Ingestion Pipeline**
    - *Subphase 5.1.1:* Markdown, PDF, plain text, and source code chunker with file hash tracking and incremental re-indexing.
  - **Phase 5.2: Local Hybrid Vector + Lexical Search & Citation Engine**
    - *Subphase 5.2.1:* Local embedding adapter (CPU-first) + cosine/BM25 reciprocal rank fusion with exact source file/line citations.
  - **Phase 5.3: Sandboxed Tool Execution & Knowledge Base UI**
    - *Subphase 5.3.1:* Capability-restricted local tool runner (allowlisted workspace paths, timeout enforcement, zero network access in `LocalOnlyPrivacyMode`).
    - *Subphase 5.3.2:* Flutter Knowledge Base manager and clickable inline citation cards in Chat UI.
- **Exit Criteria:** Offline RAG queries return verifiable file+line citations; path-escape and unauthorized tool calls are deterministically blocked.

---

## Stage 6: Vision, Image Understanding, Generation & Editing

- **Objective:** Deliver independent Vision Understanding (`Modality::Vision`) and Image Generation/Editing (`Modality::ImageGeneration`) runtime adapters and the Flutter Multimodal Vision & Canvas Studio.
- **Dependencies:** Stage 5 Gate Verified.
- **Active Skills:** `miniai-multimodal`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-flutter-ui`, `miniai-security-privacy`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 6.1: Vision Understanding Adapter (`Modality::Vision`)**
    - *Subphase 6.1.1:* Image header/bounds sanitizer (`<= 4096x4096`, magic byte verification) and CPU-first visual QA / OCR / captioning adapter.
  - **Phase 6.2: Image Generation & Inpainting Adapter (`Modality::ImageGeneration`)**
    - *Subphase 6.2.1:* Deterministic seed text-to-image and mask-based image editing pipeline with step progress callbacks and PNG provenance metadata.
  - **Phase 6.3: Flutter Vision & Image Canvas Studio**
    - *Subphase 6.3.1:* Interactive image inspection, mask brush canvas, seed/step controls, and gallery persistence.
- **Exit Criteria:** Malformed/oversized images safely rejected; vision QA and image synthesis execute via independent adapters on CPU and optional GPU.

---

## Stage 7: Speech, Audio, TTS, ASR, Voice & Music

- **Objective:** Deliver independent Speech ASR (`Modality::SpeechAsr`), Neural TTS (`Modality::SpeechTts`), real-time voice interaction, and Music Generation (`Modality::Music`) adapters and the Flutter Audio & Music Studio.
- **Dependencies:** Stage 6 Gate Verified.
- **Active Skills:** `miniai-multimodal`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-flutter-ui`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 7.1: Offline Speech Recognition (`Modality::SpeechAsr`)**
    - *Subphase 7.1.1:* WAV/FLAC/OGG audio normalizer (`16 kHz` mono PCM), voice activity detection, and timestamped transcription adapter.
  - **Phase 7.2: Neural Text-to-Speech (`Modality::SpeechTts`) & Voice Loop**
    - *Subphase 7.2.1:* Low-latency streaming TTS synthesis adapter and full-duplex offline voice conversation loop (ASR -> LLM -> TTS).
  - **Phase 7.3: Music Generation (`Modality::Music`) & Audio Studio UI**
    - *Subphase 7.3.1:* Prompt-to-music generation adapter with duration/memory guards and Flutter waveform player/recorder UI.
- **Exit Criteria:** Offline ASR and TTS round-trip tests pass on CPU; generated WAV files have valid headers and non-silent PCM frames.

---

## Stage 8: Video Understanding, Generation & Media Workflows

- **Objective:** Deliver Video Understanding and Video Generation (`Modality::Video`) via an independent adapter, temporal frame indexing, and multi-stage media workflow chaining.
- **Dependencies:** Stage 7 Gate Verified.
- **Active Skills:** `miniai-multimodal`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-evaluation-benchmarks`, `miniai-flutter-ui`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 8.1: Video Frame Extraction & Temporal Understanding**
    - *Subphase 8.1.1:* Memory-bounded keyframe sampler, audio track splitter, and temporal scene summarization pipeline.
  - **Phase 8.2: Local Video Generation & Composable Media Pipelines**
    - *Subphase 8.2.1:* Prompt/image-to-video synthesis adapter and DAG workflow runner (e.g., Script LLM -> TTS Voiceover + Image Gen -> Video Assembly).
  - **Phase 8.3: Flutter Video Timeline & Workflow Studio**
    - *Subphase 8.3.1:* Keyframe scrubber, scene transcript inspector, and workflow node monitor.
- **Exit Criteria:** Frame sampling stays strictly within configured RAM budget; end-to-end multimodal media workflow completes offline on CPU.

---

## Stage 9: Hybrid Cloud GPU & Secure Distributed Compute

- **Objective:** Deliver consent-gated hybrid cloud GPU execution and secure distributed compute node orchestration with mTLS encryption, strict privacy policy enforcement, and automatic local CPU fallback.
- **Dependencies:** Stage 8 Gate Verified.
- **Active Skills:** `miniai-security-privacy`, `miniai-offline-first`, `miniai-hardware-optimization`, `miniai-containers-dev`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-testing-qa`.
- **Phases & Subphases:**
  - **Phase 9.1: Cryptographic Cloud Consent Engine & Audit Ledger**
    - *Subphase 9.1.1:* Scoped `CloudConsentToken` issuance, expiration, instant revocation, and immutable SQLite consent audit log.
  - **Phase 9.2: Secure Remote Compute Dispatcher & Local Fallback**
    - *Subphase 9.2.1:* Encrypted remote job dispatch with payload minimization, remote attestation check, and automatic fallback to local CPU/GPU if consent is revoked or network drops.
  - **Phase 9.3: Flutter Hybrid Compute & Consent Governance UI**
    - *Subphase 9.3.1:* Explicit consent grant dialog showing exact payload bytes, target endpoint, retention policy, and one-click kill-switch (`LocalOnlyPrivacyMode`).
- **Exit Criteria:** 100% of remote compute calls are blocked when `LocalOnlyPrivacyMode == true` or token is expired/revoked; instant revocation halts active remote stream and falls back locally.

---

## Stage 10: Cross-Platform Hardening, Certification, Packaging & Releases

- **Objective:** Harden performance, security, and accessibility across Windows, Linux, macOS, Android, and iOS; produce reproducible release packages, SBOMs, SHA-256 manifests, and complete final Release Certification.
- **Dependencies:** Stage 9 Gate Verified.
- **Active Skills:** `miniai-release-certification`, `miniai-github-actions`, `miniai-security-privacy`, `miniai-evaluation-benchmarks`, `miniai-testing-qa`, `miniai-independent-audit`.
- **Phases & Subphases:**
  - **Phase 10.1: Full-System Stress, Fuzz, Offline & Security Hardening**
    - *Subphase 10.1.1:* Long-running soak tests, corrupted input fuzzing across all 7 modalities, and dependency vulnerability audit.
  - **Phase 10.2: Multi-OS Packaging & Release Automation**
    - *Subphase 10.2.1:* Windows installer/bundle, Linux AppImage/tarball, macOS `.app`/`.dmg`, Android `.apk`/`.aab`, and iOS `.ipa` (`--no-codesign` in CI) with CycloneDX SBOM and `.sha256` checksums.
  - **Phase 10.3: Final Independent Release Certification Gate**
    - *Subphase 10.3.1:* Complete execution of `RELEASE_CERTIFICATION.md` checklist and sign-off by `independent-qa-auditor` and `arch-security-reviewer`.
- **Exit Criteria:** 100% required release tests pass across all target platforms; >=90% critical business logic coverage verified; `v1.0.0` release certified.

