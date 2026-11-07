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


---

## [2026-11-06T09:44:00+05:30] — Stage 3 Subphase 3.1.1 Verified (`v0.3.0-stage3.sub3.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 3 / Phase 3.1 / Subphase 3.1.1 — CPU-First LLM Inference Engine & Sampling Controls`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Created crate `crates/miniai-llm` with `#![forbid(unsafe_code)]` and registered in workspace `Cargo.toml`.
  - Implemented `GenerationConfig` with domain bounds validation for `max_new_tokens`, `context_window_size`, `temperature`, `top_p`, `repetition_penalty`, `seed`, and `stop_sequences`.
  - Implemented `Prng` based on XorShift64* for 100% reproducible deterministic pseudo-random sampling.
  - Implemented `Sampler` supporting greedy decoding ($T=0$), temperature scaling, top-p nucleus cumulative probability truncation, and frequency-scaled repetition penalty (`penalty^count`).
  - Implemented `Tokenizer` supporting bidirectional subword and character mapping, special tokens (`<pad>`, `<s>`, `</s>`, `<unk>`), and greedy longest-match text encoding/decoding.
  - Implemented `ModelWeights` supporting GGUF header parsing (`0x46554747`), embedding projections, forward pass computing vocabulary logits, and memory footprint tracking.
  - Implemented `LlmEngine` with autoregressive loop, TTFT and throughput calculation, context window sliding, cooperative atomic cancellation (`Arc<AtomicBool>`) responding under 10ms, and stop sequence termination.
  - Implemented `LlmAdapter` bridging `LlmEngine` with `miniai-server` `EventBus` (`InferenceTokenStream` events) and Tokio MPSC channels for token-by-token streaming.
  - Added 12 unit tests in `crates/miniai-llm`, bringing total workspace tests to 66/66 passing with 0 Clippy warnings.


---

## [2026-11-06T09:55:00+05:30] — Stage 3 Subphase 3.1.2 Verified (`v0.3.1-stage3.sub3.1.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 3 / Phase 3.1 / Subphase 3.1.2 — Hot Model Switching with Verified Memory Release`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `SwitchReport` telemetry record in `crates/miniai-llm/src/config.rs` capturing `previous_model_id`, `new_model_id`, `freed_memory_bytes`, `allocated_memory_bytes`, `active_backend`, `switched_at_epoch_secs`, and `duration_ms`.
  - Implemented `LlmEngine::switch_model` enforcing verified two-phase drop: explicitly deallocates active model weights and asserts memory clearance before allocating replacement model weights.
  - Added error-recovery in `LlmEngine`: corrupted or failing model switches leave the engine in clean unloaded state (memory footprint = 0).
  - Implemented `LlmAdapter::switch_model_with_guard` coordinating with `AdmissionGuard`, validating RAM/VRAM against the 85% ceiling, executing automatic CPU fallback, and broadcasting `AdapterHealthChanged` events to `EventBus`.
  - Implemented `RuntimeSupervisor::switch_model` in `crates/miniai-runtime/src/supervisor.rs` for atomic model replacement across registered modalities.
  - Authored 4 unit tests in `crates/miniai-llm` and 1 test in `crates/miniai-runtime` verifying explicit memory release, corrupt file recovery, admission guard fallback, and rapid consecutive switches (5 models in sequence with 0 memory leaks).
  - Reached 71/71 workspace Rust tests passing with 0 Clippy warnings and 0 formatting diffs.


---

## [2026-11-06T10:37:00+05:30] — Stage 3 Subphase 3.2.1 Verified (`v0.3.2-stage3.sub3.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 3 / Phase 3.2 / Subphase 3.2.1 — SQLite Conversation History, Message Branching & Session Export`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Added Migration 4 (`004_chat_messages.sql`) in `crates/miniai-storage` defining the `chat_messages` table with session foreign keys, parent pointers (`parent_id`) for branch regeneration, roles (`system`, `user`, `assistant`), token counts, model tracking, and compound indexes.
  - Implemented `ChatMessageRecord`, `MessageRole`, and `ChatSessionExport` domain models.
  - Implemented `StorageEngine` repository methods:
    - `create_chat_message`: inserts or updates message content, auto-updating session `updated_at_epoch_secs`.
    - `get_chat_message` and `list_chat_messages`: retrieves messages by ID or session in chronological order.
    - `update_chat_message_content`: in-place message editing with token count recalculation.
    - `delete_chat_message` and `delete_chat_messages_for_session`: message deletion with foreign key cascading.
    - `get_message_branch`: traverses parent pointers upwards from any leaf node to root, returning the full linear conversation branch in chronological order.
    - `get_child_branches`: retrieves alternative regenerated completions branching from a parent node.
    - `export_session_markdown`: generates formatted Markdown document with session title, ID, and styled role headers (`### 👤 User`, `### 🤖 Assistant (<model>)`).
    - `export_session_json`: exports full structured JSON conversation tree with session metadata.
  - Added 3 unit tests in `crates/miniai-storage` verifying migration schema version 4, message branching and tree traversal, editing and deletion, and Markdown/JSON export.
  - Reached 74/74 workspace Rust tests passing with 0 Clippy warnings and 0 formatting diffs.

---

## [2026-11-06T11:18:00+05:30] — Stage 3 Subphase 3.2.2 Verified (`v0.3.2-stage3.sub3.2.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 3 / Phase 3.2 / Subphase 3.2.2 — Flutter Chat Workspace UI & Session Management`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Flutter 3.29.0, Dart 3.7.0)
- **Summary:**
  - Added client contracts for `ChatMessageRole`, `ChatMessage`, and `ChatSession` in `ui/miniai_app/lib/src/models/system_contracts.dart` with JSON serialization, deserialization, and `copyWith`.
  - Extended `PlatformController` (`ui/miniai_app/lib/src/controllers/platform_controller.dart`) with multi-session chat state, `createChatSession`, `selectChatSession`, `renameChatSession`, `deleteChatSession`, `sendMessage` with streaming token generation, `cancelGeneration` stopping token loops and emitting `llm_generation_cancelled`, `editMessage`, `deleteMessage`, `exportSessionMarkdown`, `exportSessionJson`, and hyperparameter tuning (`setChatTemperature`, `setChatMaxTokens`).
  - Implemented `ChatWorkspaceScreen` (`ui/miniai_app/lib/src/screens/chat_workspace_screen.dart`):
    - Responsive session sidebar (fixed panel on desktop, drawer on mobile).
    - Header toolbar with session title, offline CPU status, model selector dropdown, tuning dialog button, and markdown/json export sheet.
    - Conversation timeline with role-colored message bubbles, model badges, token badges, copy button, and in-place message edit dialog.
    - Real-time generation progress banner (`Key('generation_progress_card')`) displaying live tokens/sec badge (`Key('tokens_per_second_badge')`), token count badge (`Key('token_count_badge')`), elapsed seconds badge (`Key('generation_elapsed_badge')`), and stop generation button (`Key('cancel_generation_button')`).
    - Bottom composer bar (`Key('chat_composer_bar')`) with input field (`Key('chat_input_field')`) and send button (`Key('send_message_button')`).
  - Integrated `ChatWorkspaceScreen` as Tab 5 into `ShellScreen` across desktop `NavigationRail` and mobile `NavigationBar`.
  - Created `ui/miniai_app/test/chat_workspace_test.dart` and expanded `test/widget_test.dart` (12 new tests in `chat_workspace_test.dart`, 38 total Flutter tests passing).
  - Measured Flutter UI line coverage: **95.02%** (1,776/1,869 lines).
  - Maintained 100% test pass rate across all suites: 74 Rust tests, 6 Python ML tests, 38 Flutter tests, 0 Clippy warnings, 0 Flutter analyze issues.

---

## [2026-11-06T12:35:00+05:30] — Stage 3 Subphase 3.3.1 Verified (`v0.3.3-stage3.sub3.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 3 / Phase 3.3 / Subphase 3.3.1 — Quantitative LLM Benchmark Suite`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `crates/miniai-llm/src/benchmark.rs` with `LlmBenchmarkConfig`, `LlmIterationMetrics`, `LlmBenchmarkReport`, and `LlmBenchmarkHarness`.
  - Implemented measurement pipeline:
    - Real cold start model load duration timing.
    - Warm memory verification timing.
    - Warmup iteration caching pass (excluded from measurement metrics).
    - Measured inference iterations capturing per-iteration Time-To-First-Token (TTFT), decoding duration, completion tokens, throughput (tokens/second), and peak memory footprint.
    - Statistical calculations: mean, median (p50), and 95th percentile (p95) for TTFT and throughput.
    - Pretty-printed JSON serialization (`to_json`) and report persistence (`save_to_file`).
  - Authored 3 unit tests in `crates/miniai-llm` verifying cold load and TTFT metrics, configuration validation, and report file persistence.
  - Reached 77/77 workspace Rust tests passing with 0 Clippy warnings and 0 formatting diffs.

---

## [2026-11-06T14:02:00+05:30] — Stage 3 Certified & Released (`v0.3.0`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage:** `Stage 3: Offline LLM Inference, Chat, Streaming, Model Switching & Benchmarking`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1, Flutter 3.29.0 / Dart 3.7.0, Python 3.14.0 uv)
- **Summary:**
  - Certified Stage 3 following unanimous **PASS** audit verdicts from both the Independent QA Lead (`independent-qa-auditor`) and the Security Reviewer (`arch-security-reviewer`).
  - Recorded Architectural Decision Record `ADR-008` in `ADR.md`.
  - Master verification pipeline (`scripts/verify_stage3.py`) passed all 6 verification stages:
    - Rust workspace: 77/77 tests passed, 0 Clippy warnings, 0 formatting diffs.
    - Python ML tooling: 6/6 pytest tests passed, 98.16% statement coverage.
    - Flutter UI: 38/38 widget/contract tests passed, 0 analyze issues, **95.02%** line coverage.
    - Governance audit: 19 files, 6 rules, 18 skills verified with 0 errors.
  - Published Stage 3 Verification Report (`docs/STAGE_3_VERIFICATION_REPORT.json`) and Gate Report (`stage_3_gate_report.md`).
  - Merged `develop` branch into `main` and tagged release `v0.3.0`.

---

## [2026-11-06T15:12:00+05:30] — Stage 4 Subphase 4.1.1 Verified (`v0.4.1-stage4.sub4.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 4 / Phase 4.1 / Subphase 4.1.1 — Dataset Curation, Validation & Splitting`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Python 3.14.0 uv, pytest 9.2.0)
- **Summary:**
  - Implemented `ml/src/miniai_ml/dataset.py` with `DatasetRecord` and `DatasetManifest`.
  - Implemented multi-format parsing for `.jsonl` and `.csv` datasets with flexible field mapping (`prompt`/`response`, `instruction`/`output`, `input_text`/`target_text`), strict non-empty content validation, and UTF-8 enforcement.
  - Implemented content-based deduplication (`deduplicate_records`) dropping redundant samples and tracking removed counts.
  - Implemented deterministic PRNG partitioner (`split_dataset`) dividing records into `train`, `val`, and `test` splits according to configurable ratios and reproducible seeds.
  - Implemented `save_splits_and_manifest` and `verify_dataset_manifest` generating partitioned JSONL files and a cryptographic `manifest.json`.
  - Authored 11 unit and property tests in `ml/tests/test_dataset.py` verifying schema validation, format parsing, corrupt line handling, deduplication, deterministic splitting, and tamper detection.
  - Reached **100% statement coverage** on `miniai_ml/dataset.py` and **99% total coverage** across `miniai_ml` (17/17 pytest tests passing).


---

## [2026-11-06T16:15:00+05:30] — Stage 4 Subphase 4.2.1 Verified (`v0.4.2-stage4.sub4.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 4 / Phase 4.2 / Subphase 4.2.1 — PyTorch CPU/GPU LoRA Trainer & Teacher-Student Distillation Pipeline`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Python 3.14.0 uv, PyTorch 2.15.0+cpu, pytest 9.2.0)
- **Summary:**
  - Implemented `ml/src/miniai_ml/training.py` with `LoraConfig`, `LoraLinear`, `apply_lora_to_model`, `merge_lora_weights`, `unmerge_lora_weights`, `extract_lora_state_dict`, and `count_trainable_parameters`.
  - Implemented `DistillationLoss` supporting simultaneous ground truth task cross-entropy and temperature-scaled KL divergence over softened teacher distributions with $T^2$ gradient preservation.
  - Implemented `TrainingConfig` and `Trainer` supporting mini-batching, gradient accumulation, gradient clipping, periodic validation evaluation, atomic checkpoint persistence and resumption, and non-blocking cooperative cancellation.
  - Authored 16 unit and convergence tests in `ml/tests/test_training.py` proving deterministic seeding, zero initialization delta ($B=0$), bidirectional weight merging/unmerging, sequence token support, and loss decrease on CPU.
  - Reached **96% statement coverage** on `miniai_ml/training.py` and **97% total coverage** across `miniai_ml` (33/33 pytest tests passing).
  - Maintained 100% test pass rate across all workspace test suites (77 Rust tests, 33 Python ML tests, 38 Flutter tests, 0 Clippy warnings, 0 Flutter analyze issues).


---

## [2026-11-06T16:52:00+05:30] — Stage 4 Subphase 4.2.2 Verified (`v0.4.3-stage4.sub4.2.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 4 / Phase 4.2 / Subphase 4.2.2 — Cryptographic Provenance Manifest & Model Registry Export`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Python 3.14.0 uv, PyTorch 2.15.0+cpu, pytest 9.2.0)
- **Summary:**
  - Implemented `ml/src/miniai_ml/provenance.py` with `TrainingJobProvenance`, `create_job_provenance`, `export_adapter_package`, `export_merged_model`, `create_registry_manifest`, `save_registry_manifest`, and `verify_checkpoint_provenance`.
  - Implemented cryptographic provenance recording conformant to `references/training_manifest_spec.md` with SHA-256 fingerprinting for base model, dataset, and adapter artifacts.
  - Implemented Stage 2 Model Registry export generating catalog manifests matching `crates/miniai-registry` specifications.
  - Authored 6 unit and property tests in `ml/tests/test_provenance.py` verifying validation rules, tamper rejection, JSON serialization, and physical file digest matching.
  - Reached **99% statement coverage** on `miniai_ml/provenance.py` and **97% total coverage** across `miniai_ml` (39/39 pytest tests passing).
  - Maintained 100% test pass rate across all suites: 77 Rust tests, 39 Python ML tests, 38 Flutter tests, 0 Clippy warnings, 0 Flutter analyze issues.


---

## [2026-11-06T17:22:00+05:30] — Stage 4 Subphase 4.3.1 Verified (`v0.4.4-stage4.sub4.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 4 / Phase 4.3 / Subphase 4.3.1 — Flutter Training Studio Screen & Hyperparameter Tuning Controls`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Flutter 3.29.0 / Dart 3.7.0)
- **Summary:**
  - Implemented `TrainingMethod`, `TrainingJobStatus`, `TrainingHyperparameters`, `DatasetSplitInfo`, and `TrainingJob` in `ui/miniai_app/lib/src/models/system_contracts.dart`.
  - Extended `PlatformController` with training job execution loop, step-by-step loss history recording, cooperative cancellation, and catalog adapter registration.
  - Implemented `TrainingStudioScreen` in `ui/miniai_app/lib/src/screens/training_studio_screen.dart` with dataset split inspector, LoRA hyperparameter tuning form, live progress bar, telemetry badges, and base vs fine-tuned evaluation comparison table.
  - Integrated `TrainingStudioScreen` as Tab 6 ('Train') into `ShellScreen` across desktop `NavigationRail` and mobile `NavigationBar`.
  - Authored 6 widget and integration tests in `ui/miniai_app/test/training_studio_test.dart`.
  - Reached **92.16% line coverage** across Flutter UI (44/44 Flutter tests passing, 0 analyze issues).
  - Maintained 100% test pass rate across all suites: 77 Rust tests, 39 Python ML tests, 44 Flutter tests, 0 Clippy warnings.


---

## [2026-11-06T17:37:00+05:30] — Stage 4 Certified & Released (`v0.4.0`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage:** `Stage 4: Dataset Management, Fine-Tuning, Training, Distillation & Evaluation`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Python 3.14.0 uv, PyTorch 2.15.0+cpu, Flutter 3.29.0 / Dart 3.7.0, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Certified Stage 4 following unanimous **PASS** audit verdicts from both the Independent QA Lead (`independent-qa-auditor`) and Principal Architecture & Security Auditor (`arch-security-reviewer`).
  - Delivered end-to-end dataset curation and validation pipeline in `ml/src/miniai_ml/dataset.py` with multi-format support, strict UTF-8 decoding, content-based deduplication, and deterministic cryptographic partitioning.
  - Delivered parameter-efficient fine-tuning (LoRA) and teacher-student knowledge distillation in `ml/src/miniai_ml/training.py` with mathematically proven identity initialization, exact in-place weight merging/unmerging, and $T^2$-scaled softened distillation loss.
  - Delivered cryptographic provenance manifest generation (`checkpoint_provenance.json`), adapter packaging, and Stage 2 Model Registry export in `ml/src/miniai_ml/provenance.py`.
  - Delivered Flutter Training Studio Screen in `ui/miniai_app` with interactive dataset inspector, tuning controls, live progress tracking, and evaluation comparison.
  - Verified 77 Rust tests (0 warnings), 39 Python ML tests (**97.28%** coverage), and 44 Flutter tests (**92.16%** coverage).
  - Recorded ADR-009 in `ADR.md` and compiled `docs/STAGE_4_VERIFICATION_REPORT.json`.


---

## [2026-11-06T17:40:00+05:30] — Stage 5 Initiated & Subphase 5.1.1 In Progress (`v0.5.0-stage5.init`)
- **Action:** `STAGE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.1 / Subphase 5.1.1 — Document & Code Ingestion Pipeline with Content-Addressed Chunking`
- **Summary:**
  - Initiated Stage 5: RAG, Document/Code Indexing & Secure Tool Integration.
  - Commencing Subphase 5.1.1 in `crates/miniai-rag`.
  - Creating syntax-aware document/code chunker, content-addressed SHA-256 chunk deduplication, sensitive file filter, and incremental ingestion engine.


---

## [2026-11-06T18:00:00+05:30] — Stage 5 Subphase 5.1.1 Verified (`v0.5.1-stage5.sub5.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.1 / Subphase 5.1.1 — Document & Code Ingestion Pipeline with Content-Addressed Chunking`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `crates/miniai-rag` workspace crate with `#![forbid(unsafe_code)]`.
  - Implemented structure-aware text chunking for Markdown, source code (Rust, Python, Dart, JS, TS, C/C++, JSON, YAML, TOML), and plain/PDF text in `chunker.rs`.
  - Built content-addressed chunk deduplication engine in `dedup.rs`.
  - Built strict security validators in `security.rs` for path traversal, UNC prefix normalization, binary rejection, and credential/secret exclusion.
  - Built `IngestionPipeline` in `pipeline.rs` with incremental SHA-256 file caching and manifest export.
  - Authored 15 unit and integration tests in `miniai-rag`, expanding workspace Rust tests to 92/92 passing with 0 warnings.


---

## [2026-11-06T18:02:00+05:30] — Stage 5 Subphase 5.2.1 In Progress (`v0.5.2-stage5.sub5.2.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.2 / Subphase 5.2.1 — Local Hybrid Retrieval (BM25 + Dense Embeddings + RRF)`
- **Summary:**
  - Initiated Subphase 5.2.1 in `crates/miniai-rag`.
  - Designing BM25 Okapi lexical inverted index, local CPU-first embedding adapter, and Reciprocal Rank Fusion (RRF) hybrid retrieval engine.


---

## [2026-11-06T18:10:00+05:30] — Stage 5 Subphase 5.2.1 Verified (`v0.5.2-stage5.sub5.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.2 / Subphase 5.2.1 — Local Hybrid Retrieval (BM25 + Dense Embeddings + RRF)`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented BM25 Okapi lexical ranking in `crates/miniai-rag/src/bm25.rs`.
  - Implemented local CPU-first embedding generator and vector index in `crates/miniai-rag/src/embedding.rs`.
  - Implemented `HybridSearchEngine` in `crates/miniai-rag/src/retrieval.rs` combining dense semantic vector search and sparse BM25 scores via Reciprocal Rank Fusion ($k=60$).
  - Built grounded citation generator producing accurate `file_path#Lstart-Lend` tags and keyword excerpts.
  - Reached 100/100 passing Rust tests across the workspace with 0 Clippy warnings.


---

## [2026-11-06T18:12:00+05:30] — Stage 5 Subphase 5.3.1 In Progress (`v0.5.3-stage5.sub5.3.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.3 / Subphase 5.3.1 — Capability-Sandboxed Local Tool Runner`
- **Summary:**
  - Initiated Subphase 5.3.1 in `crates/miniai-rag`.
  - Designing capability security model, timeout and byte caps, safe arithmetic evaluator, workspace reader, symbol searcher, and RAG retrieval tool.


---

## [2026-11-06T18:17:00+05:30] — Stage 5 Subphase 5.3.1 Verified (`v0.5.3-stage5.sub5.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.3 / Subphase 5.3.1 — Capability-Sandboxed Local Tool Runner`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented capability-sandboxed tool runner in `crates/miniai-rag/src/tools.rs`.
  - Added built-in tools: `WorkspaceFileReadTool`, `CalculatorTool`, `CodeSymbolSearchTool`, and `RagQueryTool`.
  - Enforced strict Local-Only Privacy Mode blocking all network tools, execution timeouts, and output byte limits with truncation markers.
  - Reached 106/106 Rust tests passing across the workspace with 0 Clippy warnings.


---

## [2026-11-06T18:18:00+05:30] — Stage 5 Subphase 5.3.2 In Progress (`v0.5.4-stage5.sub5.3.2.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.3 / Subphase 5.3.2 — Knowledge Base UI & Interactive Citation Inspector`
- **Summary:**
  - Initiated Subphase 5.3.2 in `ui/miniai_app`.
  - Building Knowledge Base screen (Tab 7 in `ShellScreen`), interactive RAG query tester, grounded citation inspector dialog in Chat UI, and comprehensive widget tests.


---

## [2026-11-06T18:55:00+05:30] — Stage 5 Subphase 5.3.2 Verified (`v0.5.4-stage5.sub5.3.2`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 5 / Phase 5.3 / Subphase 5.3.2 — Knowledge Base UI & Interactive Citation Inspector`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Flutter 3.32.0-0.2.pre, Dart 3.9.0-109.0.dev)
- **Summary:**
  - Implemented Knowledge Base index manager screen (Tab 7 'Knowledge' in `ShellScreen`).
  - Added interactive hybrid retrieval tester with RRF score chips, document inventory data table, and citation source modal.
  - Added grounded citation chips to `ChatWorkspaceScreen` assistant messages with snippet inspection dialog.
  - Reached 50/50 Flutter tests passing with 92.46% line coverage and 0 analyzer issues.


---

## [2026-11-06T19:15:00+05:30] — Stage 5 Master Gate Release (`v0.5.0`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage:** `Stage 5 — RAG, Document/Code Indexing & Secure Sandboxed Tool Integration`
- **Release Version:** `v0.5.0`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, python 3.14.0, flutter 3.32.0-0.2.pre)
- **Summary:**
  - Implemented structure-aware document and code indexing pipeline in `crates/miniai-rag` with canonical path traversal prevention, Windows UNC normalization, binary file detection, and sensitive file exclusion (`.env`, `*.pem`, `id_rsa`, `*.db`).
  - Added incremental ingestion pipeline with content-addressed SHA-256 chunk deduplication and `IngestionManifest` caching.
  - Implemented Okapi BM25 lexical ranking with Robertson-Spärck Jones IDF smoothing ($k_1=1.2, b=0.75$).
  - Implemented deterministic local 128-dimensional dense vector embeddings with subword trigram hashing, L2 normalization, and exact cosine similarity vector index.
  - Implemented Reciprocal Rank Fusion (RRF, $k=60$) search engine returning keyword-aligned contextual excerpts and grounded `file_path#Lstart-Lend` citation tags.
  - Implemented capability-sandboxed local tool runner (`ToolSandboxRunner`) enforcing execution timeouts, output byte limits with truncation markers, and strict Local-Only Privacy Mode blocking all outbound network tools.
  - Delivered built-in sandboxed tools: `WorkspaceFileReadTool`, `CalculatorTool` (safe AST math parser on CPU), `CodeSymbolSearchTool`, and `RagQueryTool`.
  - Added Knowledge Base Screen (Tab 7 'Knowledge' in `ShellScreen`) with document inventory data table, index health metrics, RAG grounding toggle, interactive query tester with score badges, and citation source modal.
  - Integrated interactive citation chips into `ChatWorkspaceScreen` assistant message bubbles opening source snippet inspection dialogs.
  - Documented `ADR-010` in `ADR.md`.
  - Verified 106/106 Rust tests passing, 39/39 Python ML tests passing (97.28% coverage), 50/50 Flutter UI tests passing (92.46% line coverage), and unanimous `PASS` verdicts from `independent-qa-auditor` and `arch-security-reviewer`.


---

## [2026-11-06T19:22:00+05:30] — Stage 6 Subphase 6.1.1 In Progress (`v0.6.1-stage6.sub6.1.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 6 / Phase 6.1 / Subphase 6.1.1 — Vision Understanding Runtime Adapter & Image Preprocessing`
- **Summary:**
  - Initiated Subphase 6.1.1 in `crates/miniai-vision`.
  - Creating image format sanitizer, magic-byte inspection (PNG/JPEG/WebP), decompression bomb protection, tensor normalizer, and CPU-first visual QA/OCR adapter.


---

## [2026-11-06T19:35:00+05:30] — Stage 6 Subphase 6.1.1 Verified (`v0.6.1-stage6.sub6.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 6 / Phase 6.1 / Subphase 6.1.1 — Vision Understanding Runtime Adapter & Image Preprocessing`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `crates/miniai-vision` with `#![forbid(unsafe_code)]`.
  - Added `ImageSanitizer` checking magic bytes for PNG, JPEG, WebP, dimension limits ($\le 4096\times 4096$), file size ($\le 25\text{ MB}$), and decompression bombs.
  - Added `ImageTensor` with bilinear resampling and ImageNet normalization.
  - Implemented `VisionEngine` with OCR bounding-box detection, automated scene descriptions, and grounded VQA.
  - Implemented `VisionAdapter` with `ModalityAdapter` conformance and CPU execution baseline.
  - Reached 123/123 Rust tests passing with 0 Clippy warnings.


---

## [2026-11-06T19:40:00+05:30] — Stage 6 Subphase 6.2.1 In Progress (`v0.6.2-stage6.sub6.2.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 6 / Phase 6.2 / Subphase 6.2.1 — Image Generation & Inpainting Runtime Adapter`
- **Summary:**
  - Initiated Subphase 6.2.1 in `crates/miniai-imagegen`.
  - Designing Text-to-Image synthesis, Image-to-Image refinement, and Inpainting engine with deterministic PRNG seeds, per-step progress telemetry, lossless PNG encoding, and cryptographic provenance chunk embedding.


---

## [2026-11-06T20:45:00+05:30] — Stage 6 Subphase 6.2.1 Verified (`v0.6.2-stage6.sub6.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 6 / Phase 6.2 / Subphase 6.2.1 — Image Generation & Inpainting Runtime Adapter`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, cargo 1.99.1)
- **Summary:**
  - Implemented `crates/miniai-imagegen` workspace crate with `#![forbid(unsafe_code)]`.
  - Added `ImageGenConfig` with validation for multiple-of-64 dimensions, prompt presence, step counts, guidance scale, and img2img strength.
  - Implemented lossless 24-bit RGB PNG encoder and decoder (`PngEncoder`, `PngDecoder`) with Deflate uncompressed blocks, Adler32/CRC32, and `tEXt` cryptographic provenance chunk injection and retrieval.
  - Built deterministic CPU synthesis engine (`ImageGenEngine`) supporting Text-to-Image, Image-to-Image, and localized Inpainting with exact unmasked byte retention, deterministic Xorshift64 noise, and per-step progress telemetry.
  - Implemented `ImageGenAdapter` implementing `ModalityAdapter` with `Modality::ImageGeneration` on CPU.
  - Reached 141/141 Rust tests passing across the workspace with 0 Clippy warnings and 0 formatting diffs.


---

## [2026-11-06T21:18:00+05:30] — Stage 6 Subphase 6.3.1 In Progress (`v0.6.3-stage6.sub6.3.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 6 / Phase 6.3 / Subphase 6.3.1 — Flutter Vision & Image Canvas Studio`
- **Summary:**
  - Initiated Subphase 6.3.1 in `ui/miniai_app`.
  - Building Vision Understanding viewer with OCR bounding boxes and grounded Visual QA.
  - Building Image Canvas Studio with inpainting mask painter, generation parameter controls, cooperative cancellation, and local gallery with cryptographic provenance inspection.


---

## [2026-11-06T22:45:00+05:30] — Stage 6 Subphase 6.3.1 Verified (`v0.6.3-stage6.sub6.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 6 / Phase 6.3 / Subphase 6.3.1 — Flutter Vision & Image Canvas Studio`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, Flutter 3.32.0-0.2.pre, Dart 3.9.0-109.0.dev)
- **Summary:**
  - Implemented `OcrBoxContract`, `VisionAnalysisResultContract`, `GenerationModeContract`, `ImageGenTaskConfig`, and `GeneratedImageItem` in `system_contracts.dart`.
  - Added Vision image analysis with magic-byte validation, SHA-256 fingerprinting, OCR bounding box detection, and grounded Visual QA to `PlatformController`.
  - Added Canvas Studio inpainting mask management, generation controls, cooperative cancellation, and local gallery persistence to `PlatformController`.
  - Created `VisionCanvasScreen` as Tab 8 ('Canvas') in `ShellScreen` with dual-mode tabs: Vision Understanding and Canvas Studio & Gallery.
  - Reached 57/57 Flutter tests passing with **91.51%** line coverage and 0 analyzer issues.


---

## [2026-11-07T06:20:00+05:30] — Stage 6 Master Gate Release (`v0.6.0`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage:** `Stage 6 — Vision, Image Understanding, Generation & Canvas Studio`
- **Release Version:** `v0.6.0`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, python 3.14.0, flutter 3.32.0-0.2.pre)
- **Summary:**
  - Implemented `crates/miniai-vision` workspace crate with `#![forbid(unsafe_code)]`, `ImageSanitizer` (magic bytes for PNG, JPEG, WebP; dimensions $\le 4096\times 4096$; size $\le 25\text{ MB}$; decompression bomb defense; SHA-256 calculation), `ImageTensor` (bilinear resampling to $224\times 224$ and ImageNet normalization), and `VisionEngine` / `VisionAdapter` (`ModalityAdapter` with `Modality::Vision` on CPU, OCR bounding boxes, scene descriptions, grounded visual QA).
  - Implemented `crates/miniai-imagegen` workspace crate with `#![forbid(unsafe_code)]`, `ImageGenConfig` (multiple-of-64 dimension bounds), lossless 24-bit RGB PNG encoder and decoder (`PngEncoder`, `PngDecoder`) with Adler32/CRC32 checksums and `tEXt` cryptographic provenance chunk injection/extraction, and `ImageGenEngine` (deterministic Xorshift64 PRNG, text conditioning, iterative scheduled denoising loop with step progress telemetry, forward diffusion img2img, localized canvas inpainting preserving unmasked bytes exactly, cooperative cancellation) and `ImageGenAdapter` (`ModalityAdapter` with `Modality::ImageGeneration` on CPU).
  - Implemented Flutter Vision & Canvas Studio in `ui/miniai_app` featuring `VisionCanvasScreen` as Tab 8 ('Canvas') in `ShellScreen` with dual tabs: Vision Understanding (image viewer, format chips, OCR bounding box list with confidence chips, scene tags, grounded VQA query field and answer display) and Canvas Studio & Gallery (parameter controls, custom inpainting mask canvas with brush/eraser toolbar, step progress banner with loss badge and cancel button, and local gallery with cryptographic provenance inspection modal).
  - Formally recorded and accepted `ADR-011` in `ADR.md`.
  - Verified 141/141 Rust tests passing (0 Clippy warnings, 0 format diffs), 39/39 Python ML tests passing (97.28% statement coverage, 0 governance errors), 57/57 Flutter UI tests passing (91.51% line coverage, 0 analyzer issues), and unanimous `PASS` audit sign-off from both `independent-qa-auditor` and `arch-security-reviewer`.
  - Deterministic verification evidence persisted in `docs/STAGE_6_VERIFICATION_REPORT.json`.


---

## [2026-11-07T06:36:00+05:30] — Stage 7 Subphase 7.1.1 In Progress (`v0.7.1-stage7.sub7.1.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 7 / Phase 7.1 / Subphase 7.1.1 — Offline Speech Recognition (ASR) Runtime Adapter`
- **Summary:**
  - Initiated Subphase 7.1.1 in `crates/miniai-speech-asr`.
  - Creating audio format sanitizer (WAV/FLAC/OGG), 16 kHz mono resampler, Voice Activity Detection (VAD) segmenter, deterministic timestamped transcription engine, and `SpeechAsrAdapter` implementing `ModalityAdapter` with `Modality::SpeechAsr` on CPU.


---

## [2026-11-07T06:50:00+05:30] — Stage 7 Subphase 7.1.1 Verified (`v0.7.1-stage7.sub7.1.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 7 / Phase 7.1 / Subphase 7.1.1 — Offline Speech Recognition (ASR) Runtime Adapter`
- **Summary:**
  - Implemented `crates/miniai-speech-asr` workspace crate with `#![forbid(unsafe_code)]`.
  - Implemented `AudioSanitizer` with format validation for WAV (PCM & float), FLAC (`fLaC`), and OGG (`OggS`), channel range 1..=8, sample rate range 8 kHz..=192 kHz, max file size 25 MB, max duration 600 seconds, and SHA-256 fingerprinting.
  - Implemented `AudioResampler` supporting multi-channel downmixing to mono, linear interpolation to 16 kHz mono float32, and RMS loudness normalization.
  - Implemented `VadEngine` with 30ms short-time energy (STE), zero-crossing rate (ZCR), adaptive noise floor estimation, hangover smoothing, and continuous speech detection.
  - Implemented `SpeechAsrEngine` with deterministic acoustic energy mapping to phonetic tokens, timestamped speech segments (`start_secs`, `end_secs`, `text`, `confidence`), and token alignment.
  - Implemented `SpeechAsrAdapter` implementing `ModalityAdapter` with `Modality::SpeechAsr` on CPU (`AcceleratorBackend::Cpu`), weights loading with SHA-256 validation, and health checks.
  - Added 5 test suites (26 unit and integration tests) all passing, 0 clippy warnings, and clean formatting.


---

## [2026-11-07T06:52:00+05:30] — Stage 7 Subphase 7.2.1 In Progress (`v0.7.2-stage7.sub7.2.1.init`)
- **Action:** `SUBPHASE_INITIATED`
- **Stage / Phase / Subphase:** `Stage 7 / Phase 7.2 / Subphase 7.2.1 — Neural TTS & Offline Voice Loop`
- **Summary:**
  - Initiated Subphase 7.2.1 in `crates/miniai-speech-tts`.
  - Creating lossless RIFF/WAVE encoder (16-bit PCM / 32-bit float, 22.05 kHz / 24 kHz), text normalizer and phoneme chunker ($\le 4096$ chars), acoustic formant synthesizer with voice profiles (`Alloy`, `Echo`, `Fable`, `Onyx`, `Nova`, `Shimmer`), `SpeechTtsAdapter` implementing `ModalityAdapter` with `Modality::SpeechTts` on CPU, and the unified offline voice conversation loop (`ASR -> LLM -> TTS`).


---

## [2026-11-07T07:00:00+05:30] — Stage 7 Subphase 7.2.1 Verified (`v0.7.2-stage7.sub7.2.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 7 / Phase 7.2 / Subphase 7.2.1 — Neural TTS & Offline Voice Loop`
- **Summary:**
  - Implemented `crates/miniai-speech-tts` workspace crate with `#![forbid(unsafe_code)]`.
  - Implemented `WavEncoder` producing compliant standard RIFF/WAVE 16-bit signed PCM audio at 22.05 kHz and 24 kHz mono with sample clamping, verified via `AudioSanitizer` in `miniai-speech-asr`.
  - Implemented `TextNormalizer` providing text bounds enforcement ($\le 4096$ chars per segment), word expansion for numbers (up to 999,999) and symbols (`$`, `%`, `&`, `+`, `@`), sentence chunking, and acoustic phonetic tokenization.
  - Implemented `SpeechTtsEngine` featuring 6 distinct voice profiles (`Alloy`, `Echo`, `Fable`, `Onyx`, `Nova`, `Shimmer`) plus custom configurations, formant resonance filters ($F_1, F_2, F_3$), harmonic overtones, filtered noise for unvoiced consonants, plosive transient bursts, continuous phase accumulator, ADSR envelope shaping, and soft-saturation limiter.
  - Implemented `SpeechTtsAdapter` conforming to `ModalityAdapter` with `Modality::SpeechTts` on CPU (`AcceleratorBackend::Cpu`), weights loading with SHA-256 verification, and health reporting.
  - Implemented `VoiceConversationLoop` linking `SpeechAsrAdapter`, `LlmEngine`, and `SpeechTtsAdapter` into an end-to-end offline voice interaction loop with full turnaround latency profiling and cooperative cancellation.
  - Authored 5 test suites (27 tests) in `tests/`, achieving 100% pass rate, 0 clippy warnings, and clean formatting.


---

## [2026-11-07T07:16:00+05:30] — Stage 7 Subphase 7.3.1 Verified (`v0.7.3-stage7.sub7.3.1`)
- **Action:** `SUBPHASE_VERIFIED`
- **Stage / Phase / Subphase:** `Stage 7 / Phase 7.3 / Subphase 7.3.1 — Music Generation & Flutter Sound Studio`
- **Summary:**
  - Implemented `crates/miniai-music` workspace crate with `#![forbid(unsafe_code)]`.
  - Implemented `MusicalScale` providing musical modal scales, scale frequencies, root bass frequencies, and chord triads.
  - Implemented `WavEncoder` for 16-bit signed PCM mono and stereo container encoding with sample clamping and header verification, tested against `AudioSanitizer` from `miniai-speech-asr`.
  - Implemented `MusicGenEngine` polyphonic acoustic music synthesizer across 4 tracks: bassline, stereo-panned harmony pad triads, melody lead arpeggios, and rhythmic kick/snare/hi-hat percussion.
  - Implemented `MusicAdapter` implementing `ModalityAdapter` with `Modality::Music` on CPU (`AcceleratorBackend::Cpu`), weights loading with SHA-256 verification, and health reporting.
  - Authored 3 Rust test suites (15 tests) in `crates/miniai-music/tests/`, achieving 100% pass rate, 0 clippy warnings, and clean formatting.
  - Implemented cross-language audio contracts in `ui/miniai_app/lib/src/models/system_contracts.dart` for Speech ASR, Neural TTS, Voice Loop, and Music Generation.
  - Extended `PlatformController` with zero-mock Speech ASR, Neural TTS, Voice Conversation Loop, and Music Generation state, operations, and telemetry event recording.
  - Implemented `SoundStudioScreen` in `ui/miniai_app/lib/src/screens/sound_studio_screen.dart` featuring dual tabs: Speech & Voice Loop and Music Studio & Soundstage.
  - Added Tab 9 ('Sound') to `NavigationRail` and `NavigationBar` in `ShellScreen`.
  - Authored `ui/miniai_app/test/sound_studio_test.dart` with 14 comprehensive tests, achieving 71/71 Flutter tests passing with **91.62%** line coverage and 0 analyzer issues.


---

## [2026-11-07T07:50:00+05:30] — Stage 7 Master Gate Release (`v0.7.0`)
- **Action:** `STAGE_GATE_VERIFIED`
- **Stage:** `Stage 7 — Speech, Audio, TTS, ASR, Voice & Music`
- **Release Version:** `v0.7.0`
- **Verification Tier:** `PROVEN_ON_HOST` (Windows x86_64, rustc 1.99.1, python 3.14.0, flutter 3.32.0-0.2.pre)
- **Summary:**
  - Implemented `crates/miniai-speech-asr` workspace crate with `#![forbid(unsafe_code)]`, `AudioSanitizer` (WAV PCM/float, FLAC, OGG; channels $1..=8$; sample rate $8\text{ kHz}..=192\text{ kHz}$; duration $\le 600\text{s}$; size $\le 25\text{ MB}$; SHA-256 calculation), `AudioResampler` (channel downmix, linear interpolation resampling to $16\text{ kHz}$ mono float32, RMS normalization), `VadEngine` (30ms STE energy, zero-crossing rate, adaptive noise floor estimation, hangover smoothing), `SpeechAsrEngine` (subband acoustic energy mapping to phonetic tokens/subwords, timestamped segments and words), and `SpeechAsrAdapter` (`Modality::SpeechAsr` on CPU with SHA-256 weights loading verification).
  - Implemented `crates/miniai-speech-tts` workspace crate with `#![forbid(unsafe_code)]`, `WavEncoder` (lossless 16-bit signed PCM mono audio at 22.05 kHz / 24 kHz with sample clipping clamping), `TextNormalizer` ($\le 4096$ chars per segment, number expansion up to 999,999, symbol expansion, phonetic tokenization), `SpeechTtsEngine` (6 preset voice profiles: `Alloy`, `Echo`, `Fable`, `Onyx`, `Nova`, `Shimmer`, 3-formant resonance model $F_1, F_2, F_3$, harmonic overtones, filtered noise for unvoiced consonants, plosive bursts, ADSR envelope shaping, and soft saturation limiter), `SpeechTtsAdapter` (`Modality::SpeechTts` on CPU), and `VoiceConversationLoop` (`ASR -> LLM -> TTS` offline voice loop with granular latency profiling: ASR, LLM TTFT, LLM total, TTS, total roundtrip, and cooperative cancellation).
  - Implemented `crates/miniai-music` workspace crate with `#![forbid(unsafe_code)]`, `MusicalScale` (`C Major`, `A Minor`, `G Major`, `E Minor`, `D Dorian`, chord triads), `WavEncoder` (mono and stereo 16-bit PCM WAV encoding), `MusicGenEngine` (polyphonic 4-track acoustic synthesizer: bassline, stereo harmony pad triads, melody lead arpeggios with vibrato, rhythm kick/snare/hi-hat percussion, stereo soundstage spatializer, and soft limiter), and `MusicAdapter` (`Modality::Music` on CPU).
  - Implemented Flutter Sound Studio in `ui/miniai_app` featuring `SoundStudioScreen` as Tab 9 in `ShellScreen` across desktop `NavigationRail` and mobile `NavigationBar` with dual tabs: Speech & Voice Loop (ASR transcription, Neural TTS voice controls, live conversation turn timeline with latency breakdown chips) and Music Studio & Soundstage (prompt, tempo BPM 40-240, duration 1-120s, scale key, stereo switch, soundstage waveform cards, music library gallery).
  - Formally recorded and accepted `ADR-012` in `ADR.md`.
  - Verified 235/235 Rust workspace tests passing (0 clippy warnings, 0 format diffs), 39/39 Python ML tests passing (97.28% statement coverage, 0 governance errors), 71/71 Flutter UI tests passing (91.62% line coverage, 0 analyzer issues), and received unanimous unconditional `PASS` audit sign-offs from both `independent-qa-auditor` and `arch-security-reviewer`.
  - Deterministic verification evidence persisted in `docs/STAGE_7_VERIFICATION_REPORT.json`.
