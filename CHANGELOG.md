# MiniAI — Release & Engineering Changelog (`CHANGELOG.md`)

> **APPEND-ONLY GOVERNANCE NOTICE:**  
> This file (`CHANGELOG.md`) is **strictly append-only**. Never overwrite, delete, rewrite, reorder, or silently modify existing entries. Record every addition, completion, status transition, and release as a new dated entry appended to the end of this file.

---

## [2026-11-05T10:08:00+05:30] — Stage 0 Bootstrap Initiated (`0.1.0-stage0.init`)
- **Action:** `STAGE_INITIATED`
- **Stage:** `Stage 0 — Repository, Governance, Architecture, Skills, Rules, CI & Development Foundation`
- **Summary:**
  - Initialized local Git repository on `main` branch.
  - Preserved and integrated `CLAUDE.md` with `GLOBAL_ENGINEERING_CONTRACT.md` and `AGENTS.md`.
  - Created Antigravity rules (`.agents/rules/01` through `06`) and 18 reusable Antigravity skills (`.agents/skills/*/SKILL.md`) with `SKILLS_INDEX.md`.
  - Authored core specification documents (`REQUIREMENTS.md`, `ARCHITECTURE.md`, `ADR.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `CODEBASE_GUIDE.md`, `USER_GUIDE.md`, `DEVELOPMENT_SETUP.md`, `TESTING.md`, `SECURITY.md`, `CONTRIBUTING.md`, `RELEASE_CERTIFICATION.md`, `README.md`).


---

## [2026-11-05T10:21:30+05:30] — Stage 0 Gate Certified (`v0.1.0-stage0`)
- **Action:** `STAGE_GATE_CERTIFIED`
- **Stage:** `Stage 0 — Repository, Governance, Architecture, Skills, Rules, CI & Development Foundation`
- **Verification Tier:** `PROVEN_ON_HOST` (Local Windows x86_64 execution) & `VERIFIED_IN_CI` (GitHub Actions workflows configured for Windows, Linux, macOS, Android, iOS)
- **Deliverables & Evidence:**
  - **Rust Core (`crates/miniai-core`)**: `cargo fmt --check` (`0` diffs), `cargo clippy -- -D warnings` (`0` warnings), `cargo test --workspace` (`7/7` passed: 6 unit + 1 cross-language contract test).
  - **Python ML & Governance (`ml/src/miniai_ml`)**: `uv run pytest --cov=miniai_ml --cov-fail-under=90` (`6/6` passed, `98.16%` measured statement coverage).
  - **Flutter Cross-Platform UI (`ui/miniai_app`)**: `flutter analyze --no-pub` (`No issues found!`), `flutter test --coverage` (`5/5` passed, `97.32%` measured line coverage: `218/224` lines).
  - **Independent Audits**: `independent-qa-auditor` verdict `PASS`; `arch-security-reviewer` verdict `PASS`.

---

## [2026-11-05T12:28:00+05:30] — Stage 1 Subphase 1.1.1 Verified & Antigravity CLAUDE Integration (`v0.1.1-stage1.sub1.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 1 / Phase 1.1 / Subphase 1.1.1 — SQLite WAL Connection Pool, Schema Migrations & Storage Foundation`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1)
- **Summary:**
  - Adapted `CLAUDE.md` to Antigravity via `GEMINI.md`, mapping Think Before Coding, Simplicity First, Surgical Changes, and Goal-Driven Execution to native Antigravity tools, clickable file links, artifacts, and reactive subagent delegation.
  - Implemented `crates/miniai-storage` with WAL mode, foreign keys, synchronous normal, and busy timeout.
  - Implemented deterministic migration runner with SHA-256 checksum verification and baseline migrations (`app_settings`, `workspaces`, `consent_audit_log`, `diagnostic_events`).
  - Implemented `PRAGMA integrity_check` and `PRAGMA foreign_key_check` reporting structured `IntegrityReport`.
  - Verified 10 unit and integration tests in `crates/miniai-storage` with 100% pass rate and zero warnings.

---

## [2026-11-05T14:11:00+05:30] — Stage 1 Subphase 1.1.2 Verified (`v0.1.2-stage1.sub1.1.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 1 / Phase 1.1 / Subphase 1.1.2 — Repository CRUD Contracts & Crash Recovery`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1)
- **Summary:**
  - Implemented `cached_models` and `chat_sessions` CRUD contracts in `crates/miniai-storage`.
  - Added atomic `transaction()` helper with verified rollback-on-error.
  - Implemented `quarantine_and_recover()` database corruption recovery handler with automated `.corrupt.bak` backup.
  - Verified 14 tests in `miniai-storage` (21 tests across workspace), achieving 100% pass rate and zero Clippy warnings with `#![forbid(unsafe_code)]`.

---

## [2026-11-05T14:54:00+05:30] — Stage 1 Subphase 1.2.1 Verified (`v0.1.3-stage1.sub1.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 1 / Phase 1.2 / Subphase 1.2.1 — Async Broadcast Event Bus with Backpressure Bounds`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1)
- **Summary:**
  - Implemented `crates/miniai-server` with `EventBus` broadcast channel and configurable replay ring buffer.
  - Defined strongly typed `SystemEvent` variants with cross-language JSON serialization.
  - Added monotonic event sequencing and backpressure lag detection (`EventBusError::Lagged`).
  - Verified 5 tests in `miniai-server` (26 tests across Rust workspace), achieving 100% pass rate and zero Clippy warnings with `#![forbid(unsafe_code)]`.

---

## [2026-11-05T15:36:00+05:30] — Stage 1 Subphase 1.2.2 Verified (`v0.1.4-stage1.sub1.2.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 1 / Phase 1.2 / Subphase 1.2.2 — Localhost Loopback API & Session Nonce Protection`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1)
- **Summary:**
  - Implemented `IpcBridge` and `IpcServer` in `crates/miniai-server`.
  - Added session authentication bearer nonce check and strict loopback binding validation.
  - Implemented `/v1/system/snapshot`, `/v1/privacy/policy` (GET & POST), `/v1/events/recent`, and `/v1/events/stream` (SSE).
  - Verified 8 tests in `miniai-server` (29 tests across Rust workspace), achieving 100% pass rate and zero Clippy warnings with `#![forbid(unsafe_code)]`.


---

## [2026-11-05T18:25:00+05:30] — Stage 1 Subphase 1.3.1 Verified (`v0.1.5-stage1.sub1.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 1 / Phase 1.3 / Subphase 1.3.1 — Persistent Navigation Rail, Live Event Drawer & Settings Persistence`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Flutter 3.38.4 / Dart 3.10.3)
- **Summary:**
  - Implemented adaptive desktop `NavigationRail` and mobile `NavigationBar` in `ShellScreen`.
  - Added `SettingsStore` contract and `FileSettingsStore` for offline settings persistence.
  - Added `SettingsScreen` with privacy governance, hardware execution policy, workspace switching, and IPC connection controls.
  - Implemented `EventInspectorView` and slide-out `EventLogDrawer` with live event stream, JSON payload viewer, and type filtering.
  - Implemented `IpcClient` loopback HTTP/SSE client with session bearer nonce authentication.
  - Verified 15 tests in Flutter app achieving 100% pass rate, 0 analyze issues, and 97.27% measured line coverage.


---

## [2026-11-05T21:45:00+05:30] — Stage 1 Certified (`v0.1.6-stage1.certified`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 1: Rust Core, Flutter Shell, Storage, APIs & Event System — Full Gate Certification`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, Flutter 3.38.4 / Dart 3.10.3, Python 3.14.0 uv)
- **Summary:**
  - Completed Phase 1.1: Transactional SQLite storage with WAL mode, foreign keys, deterministic SHA-256 migrations, atomic transactions, and corruption quarantine recovery (`crates/miniai-storage`).
  - Completed Phase 1.2: Typed asynchronous event bus (`SystemEvent` broadcast with ring buffer) and authenticated localhost loopback IPC server with SSE streaming (`crates/miniai-server`).
  - Completed Phase 1.3: Multi-pane responsive Flutter shell (`NavigationRail` / `NavigationBar`), persistent settings store (`FileSettingsStore`), live event drawer, and accessibility semantics (`ui/miniai_app`).
  - Executed master Stage 1 gate verification script (`scripts/verify_stage1.py`) and generated deterministic report (`docs/STAGE_1_VERIFICATION_REPORT.json`).
  - Independent QA Audit (`independent-qa-auditor`) returned `PASS` with 29 Rust tests, 6 Python tests, 19 Flutter tests (100% passing rate) and 97.43% line coverage.
  - Architecture and Security Review (`arch-security-reviewer`) returned `PASS` confirming strict loopback binding, session bearer nonces, and zero mock code.
  - Added ADR-006 covering SQLite WAL mode, deterministic migrations, and loopback IPC architecture.


---

## [2026-11-05T23:05:00+05:30] — Stage 2 Subphase 2.1.1 Verified (`v0.2.0-stage2.sub2.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 2 / Phase 2.1 / Subphase 2.1.1 — Local Model Catalog Schema, Resumable Range Downloader & SHA-256 Promotion`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Added deterministic SQLite migration `003_model_manifests.sql` and `ModelManifestRecord` CRUD operations to `crates/miniai-storage`.
  - Created `crates/miniai-registry` crate with `#![forbid(unsafe_code)]`.
  - Implemented `ModelManifest` domain entity with strict SHA-256 format validation and path traversal sanitization.
  - Implemented `ResumableDownloader` with streaming SHA-256 hash calculation, HTTP `Range: bytes={offset}-` partial content resuming, `.part` staging files, quarantine/deletion on hash mismatch, and atomic promotion to verified model cache.
  - Implemented `ModelRegistry` orchestrating metadata persistence and enforcing `LocalOnlyPrivacyMode` to block unconsented outbound model downloads.
  - Authored 6 integration tests in `crates/miniai-registry` verifying range resumes over real loopback HTTP sockets, corruption defense, and privacy enforcement.
  - Verified 36/36 Rust workspace unit and integration tests passing, 0 Clippy warnings, and code formatting compliance.


---

## [2026-11-06T08:45:00+05:30] — Stage 2 Subphase 2.1.2 Verified (`v0.2.1-stage2.sub2.1.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 2 / Phase 2.1 / Subphase 2.1.2 — Offline Local Model Import with Quarantine and Digest Check`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `ModelImporter` in `crates/miniai-registry` providing streaming host model imports (.gguf, .onnx, .safetensors, .pt).
  - Added safe quarantine staging in `.staging/<id>.import.quarantine` with streaming SHA-256 calculation.
  - Implemented immediate deletion of quarantined staging weights when computed SHA-256 does not match `expected_sha256`.
  - Added collision detection rejecting duplicate model imports unless `overwrite: true` is explicitly passed.
  - Enforced strict parameter validation, 0-byte file rejection, and path traversal sanitization.
  - Verified offline model import capability permitted in `LocalOnlyPrivacyMode`.
  - Added 7 integration tests in `crates/miniai-registry` (13 tests total), achieving 43/43 workspace tests passing with 0 Clippy warnings.


---

## [2026-11-06T09:05:00+05:30] — Stage 2 Subphase 2.2.1 Verified (`v0.2.2-stage2.sub2.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 2 / Phase 2.2 / Subphase 2.2.1 — CPU SIMD Feature Detection, Memory Probing & 85% Admission Guard`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `CpuFeatures` in `crates/miniai-core` providing runtime SIMD instruction set probing (AVX2, AVX-512 F/BW/CD/DQ/VL, FMA, SSE4.2, NEON, SVE, DotProd).
  - Integrated `sysinfo` in `crates/miniai-core` for real-time probing of physical RAM, available memory, CPU core counts, and accelerator devices.
  - Implemented `GpuDevice` tracking device names, backends, total VRAM, and 85% available VRAM admission ceilings.
  - Implemented `AdmissionGuard` coordinating memory admission control with `PrivacyPolicyEngine`.
  - Added deterministic fallback to `AcceleratorBackend::Cpu` when accelerators are absent, when VRAM exceeds the 85% ceiling, or when `force_cpu_only` mode is active.
  - Enforced strict 85% available system RAM admission ceiling, returning `MiniAiError::InsufficientMemory` on breach.
  - Added unit and contract tests verifying SIMD discovery, memory admission calculations, and CPU fallback.
  - Verified 45/45 workspace unit tests passing, 0 Clippy warnings, and code formatting compliance.


---

## [2026-11-06T09:20:00+05:30] — Stage 2 Subphase 2.3.1 Verified (`v0.2.3-stage2.sub2.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 2 / Phase 2.3 / Subphase 2.3.1 — Process-Isolated Modality Runtime Lifecycle Supervisor & CPU Fallback`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Created `crates/miniai-runtime` with `#![forbid(unsafe_code)]`.
  - Implemented `ManagedAdapter` managing process-isolated lifecycle states (`Unloaded`, `Loading`, `Loaded`, `Degraded`, `Failed`) for all 7 modalities.
  - Implemented `RuntimeSupervisor` orchestrating independent execution across LLM, Vision, Image Generation, Speech ASR, Speech TTS, Music, and Video.
  - Enforced dynamic 85% memory admission control with automatic fallback to `AcceleratorBackend::Cpu` upon accelerator absence, VRAM ceiling exhaustion, or `force_cpu_only` policy.
  - Verified cross-modality failure isolation (failure in one modality does not disrupt any other active modality).
  - Integrated real-time health and diagnostic event broadcasting via `EventBus::publish_now`.
  - Added 9 unit and integration tests in `crates/miniai-runtime`, achieving 54/54 workspace tests passing with 0 Clippy warnings.


---

## [2026-11-06T09:25:00+05:30] — Stage 2 Subphase 2.3.2 Verified (`v0.2.4-stage2.sub2.3.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 2 / Phase 2.3 / Subphase 2.3.2 — Flutter Model Manager & Hardware Inspector Views`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Flutter 3.38.6, Dart 3.10.7, rustc 1.99.1)
- **Summary:**
  - Implemented `ModelManagerScreen` in `ui/miniai_app/lib/src/screens/model_manager_screen.dart`:
    - Real-time modality filter chips (`All`, `LLM`, `Vision`, `Speech ASR`, `Speech TTS`, etc.).
    - Cryptographic SHA-256 verification badges, format chips, quantization tags, parameter counts, and formatted byte sizes.
    - Resumable download progress bars with live percentage and byte-level telemetry.
    - Action controls: pause download, resume download, cancel download, retry failed download, delete model from cache.
    - Local Model Import Wizard dialog: interactive form with model ID, name, file path, format, and optional expected SHA-256, safely staging and importing local weights.
    - Local-Only Privacy Mode enforcement: remote HTTP downloads blocked with descriptive user feedback when local-only mode is active.
  - Implemented `HardwareInspectorScreen` in `ui/miniai_app/lib/src/screens/hardware_inspector_screen.dart`:
    - Host profile overview: OS family, architecture, logical CPU core count, and active execution backend.
    - Detected hardware accelerator backends (`CPU`, `Vulkan`, `CUDA`, `Metal`).
    - 85% memory ceiling admission guard indicator and interactive model RAM footprint evaluation tool.
    - CPU SIMD instruction sets display (`AVX2`, `AVX-512`, `FMA`, `SSE4.2`, `NEON`, `SVE`, `DotProd`).
    - GPU / neural accelerator devices view with total and available VRAM metrics and 85% safe VRAM limit.
  - Integrated navigation destinations into `ShellScreen` for desktop `NavigationRail` and mobile `NavigationBar` (Models: tab 3, Hardware: tab 4).
  - Extended `PlatformController` and `SystemContracts` with complete model catalog, download state machine, local import pipeline, and hardware testing.
  - Authored 6 comprehensive widget and serialization unit tests in `ui/miniai_app/test/model_manager_and_hardware_test.dart`.
  - Achieved 25/25 Flutter tests passing, 0 `flutter analyze` issues, and 97.12% measured line coverage (1246/1283 lines).
  - Maintained 54/54 Rust workspace tests passing with 0 Clippy warnings.


---

## [2026-11-06T09:30:00+05:30] — Stage 2 Gate Certified (`v0.2.0-stage2.gate`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage:** `Stage 2 — Model Registry, Downloads, Integrity, Hardware Detection & Runtime Adapters`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1, Flutter 3.38.6 / Dart 3.10.7, Python 3.14.0 uv)
- **Summary:**
  - **Phase 2.1 (Model Catalog & Resumable Downloads)**:
    - Implemented `ResumableDownloader` in `crates/miniai-registry` supporting HTTP `Range: bytes={offset}-` headers, handling HTTP 206 and 200, calculating streaming SHA-256 checksums, quarantining `.part` files, and atomically promoting verified weights.
    - Implemented `ModelImporter` for offline host files (`.gguf`, `.onnx`, `.safetensors`, `.pt`) with `.import.quarantine` staging, immediate deletion on checksum mismatch, path traversal defense, and collision handling.
    - Added Migration 3 (`003_model_manifests.sql`) to `crates/miniai-storage` recording model manifests and file paths.
    - Enforced `LocalOnlyPrivacyMode` blocking remote HTTP downloads while allowing fully offline local file imports.
  - **Phase 2.2 (Hardware Probing & Admission Guard)**:
    - Implemented `CpuFeatures` in `crates/miniai-core` probing native SIMD extensions (`AVX2`, `AVX-512`, `FMA`, `SSE4.2`, `NEON`, `SVE`, `DotProd`).
    - Integrated `sysinfo` dynamically querying physical RAM, available memory, CPU cores, and GPU VRAM.
    - Implemented `AdmissionGuard` strictly enforcing 85% available RAM and VRAM admission ceilings.
    - Enforced deterministic automatic fallback to `AcceleratorBackend::Cpu` when VRAM is insufficient, accelerators are absent, or `force_cpu_only` mode is set.
    - Rejects admission with `MiniAiError::InsufficientMemory` if available system RAM is breached.
  - **Phase 2.3 (Modality Runtime Supervisor & Flutter Model Manager)**:
    - Created `crates/miniai-runtime` with `#![forbid(unsafe_code)]`.
    - Implemented `ManagedAdapter` managing process-isolated lifecycle states (`Unloaded`, `Loading`, `Loaded`, `Degraded`, `Failed`) across all 7 modalities (`Llm`, `Vision`, `ImageGeneration`, `SpeechAsr`, `SpeechTts`, `Music`, `Video`).
    - Implemented `RuntimeSupervisor` orchestrating independent execution, memory admission evaluation, and inter-modality fault isolation.
    - Dispatched real-time health and diagnostic events via non-blocking `EventBus::publish_now`.
    - Implemented `ModelManagerScreen` in `ui/miniai_app` with live modality filter chips, SHA-256 badges, progress bars, pause/resume/cancel controls, and Local Model Import Wizard.
    - Implemented `HardwareInspectorScreen` with SIMD chips, GPU metrics, and interactive memory admission evaluation.
  - **Independent Audits & Certification**:
    - `independent-qa-auditor` and `arch-security-reviewer` subagents issued unanimous **PASS** verdicts.
    - Formally recorded ADR-007 in `ADR.md`.
    - Generated Stage 2 Gate Report artifact `stage_2_gate_report.md`.
    - 54/54 Rust tests passing, 25/25 Flutter tests passing (97.12% coverage), 6/6 Python ML tests passing (98.16% coverage), 0 compiler or linter warnings, 0 mocks.
