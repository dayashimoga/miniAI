# MiniAI — Engineering Task Log (`TODO.md`)

> **APPEND-ONLY GOVERNANCE NOTICE:**  
> This file (`TODO.md`) is **strictly append-only**. Never overwrite, delete, rewrite, reorder, or modify existing entries or lines. Record every task addition, status transition (`PLANNED` -> `IN_PROGRESS` -> `COMPLETED` / `VERIFIED`), and stage backlog update as a new chronologically dated entry appended to the end of this file.

---

### [2026-11-05T10:08:00+05:30] Stage 0 — Entry 001 (Master Roadmap & Stage 0 Backlog Registration)
- **Entry Type:** `INITIAL_REGISTRATION`
- **Stage / Phase:** `Stage 0 (All Phases) & Master Roadmap Stages 1–10`
- **Registered Items:**
  - `S0-P1-T1` [`IN_PROGRESS`]: Preserve supplied `CLAUDE.md` and integrate MiniAI engineering contract rules.
  - `S0-P1-T2` [`IN_PROGRESS`]: Create governance and architecture documentation suite (`GLOBAL_ENGINEERING_CONTRACT.md`, `AGENTS.md`, `REQUIREMENTS.md`, `ARCHITECTURE.md`, `IMPLEMENTATION_PLAN.md`, `CODEBASE_GUIDE.md`, `USER_GUIDE.md`, `DEVELOPMENT_SETUP.md`, `TESTING.md`, `SECURITY.md`, `CONTRIBUTING.md`, `ADR.md`, `ROADMAP.md`, `RELEASE_CERTIFICATION.md`, `README.md`, `TODO.md`, `CHANGELOG.md`).
  - `S0-P1-T3` [`IN_PROGRESS`]: Create detailed stage technical prompts (`docs/stage_prompts/STAGE_00_PROMPT.md` through `STAGE_10_PROMPT.md`).
  - `S0-P1-T4` [`IN_PROGRESS`]: Create Antigravity workspace rules in `.agents/rules/*.md`.
  - `S0-P1-T5` [`IN_PROGRESS`]: Create 18 reusable Antigravity skills in `.agents/skills/*/SKILL.md` with references, examples, and `SKILLS_INDEX.md`.
  - `S0-P2-T1` [`PLANNED`]: Implement Rust core foundation crate (`crates/miniai-core`) with modality adapters, CPU-first hardware probing, SHA-256 verification, and privacy/consent engine.
  - `S0-P2-T2` [`PLANNED`]: Write and execute Rust unit and cross-language contract tests (`>=90%` coverage).
  - `S0-P2-T3` [`PLANNED`]: Implement Flutter cross-platform UI foundation (`ui/miniai_app`) with live privacy controls, hardware admission view, and modality registry cards.
  - `S0-P2-T4` [`PLANNED`]: Write and execute Flutter static analysis and widget/semantics tests (`>=90%` model/controller coverage).
  - `S0-P2-T5` [`PLANNED`]: Implement Python ML & governance package (`ml/src/miniai_ml`) and `scripts/verify_stage0.py`.
  - `S0-P2-T6` [`PLANNED`]: Write and execute Python `pytest` suite (`>=90%` coverage).
  - `S0-P3-T1` [`PLANNED`]: Create containerized development files (`Dockerfile`, `docker-compose.yml`, `.devcontainer/devcontainer.json`).
  - `S0-P3-T2` [`PLANNED`]: Create GitHub Actions multi-OS workflows (`.github/workflows/ci.yml`, `security.yml`, `release.yml`) for Windows, Linux, macOS, Android, and iOS.
  - `S0-P3-T3` [`PLANNED`]: Integrate GitHub remote repository (`dayashimoga/miniAI`), push branches, configure branch protection where available, and verify remote state.
  - `S0-P3-T4` [`PLANNED`]: Execute independent QA (`independent-qa-auditor`) and architecture/security (`arch-security-reviewer`) audits for Stage 0 Gate sign-off.
  - `STAGE-01-BACKLOG` [`PLANNED`]: Stage 1 — Rust Core, Flutter Shell, SQLite Storage, IPC APIs & Typed Event System.
  - `STAGE-02-BACKLOG` [`PLANNED`]: Stage 2 — Model Registry, Resumable Downloads, SHA-256 Integrity, Hardware Detection & Runtime Adapters.
  - `STAGE-03-BACKLOG` [`PLANNED`]: Stage 3 — Offline LLM Inference, Chat, Token Streaming, Model Switching & Benchmarking.
  - `STAGE-04-BACKLOG` [`PLANNED`]: Stage 4 — Dataset Management, LoRA/QLoRA Fine-Tuning, Training, Distillation & Evaluation.
  - `STAGE-05-BACKLOG` [`PLANNED`]: Stage 5 — Local RAG, Document/Code Indexing & Secure Sandboxed Tool Integration.
  - `STAGE-06-BACKLOG` [`PLANNED`]: Stage 6 — Vision Understanding, Image Generation & Inpainting/Editing.
  - `STAGE-07-BACKLOG` [`PLANNED`]: Stage 7 — Speech, Audio, Neural TTS, Whisper ASR, Voice Loop & Music Generation.
  - `STAGE-08-BACKLOG` [`PLANNED`]: Stage 8 — Video Understanding, Generation & Multimodal Media Workflows.
  - `STAGE-09-BACKLOG` [`PLANNED`]: Stage 9 — Consent-Gated Hybrid Cloud GPU & Secure Distributed Compute.
  - `STAGE-10-BACKLOG` [`PLANNED`]: Stage 10 — Cross-Platform Hardening, Certification, Packaging & Releases.


---

### [2026-11-05T10:21:30+05:30] Stage 0 — Entry 002 (Stage 0 Tasks Completed & Independently Verified)
- **Entry Type:** `STATUS_TRANSITION`
- **Stage / Phase:** `Stage 0 / Phases 0.1, 0.2, 0.3 — Complete Stage 0 Gate Verification`
- **Status Transitions:**
  - `S0-P1-T1`: `IN_PROGRESS` -> `VERIFIED` — Preserved `CLAUDE.md` and integrated with `GLOBAL_ENGINEERING_CONTRACT.md` and `AGENTS.md`.
  - `S0-P1-T2`: `IN_PROGRESS` -> `VERIFIED` — Authored and verified all 18 governance and architecture documents.
  - `S0-P1-T3`: `IN_PROGRESS` -> `VERIFIED` — Authored and verified `docs/stage_prompts/STAGE_00_PROMPT.md` through `STAGE_10_PROMPT.md`.
  - `S0-P1-T4`: `IN_PROGRESS` -> `VERIFIED` — Authored and verified 6 Antigravity workspace rules in `.agents/rules/*.md`.
  - `S0-P1-T5`: `IN_PROGRESS` -> `VERIFIED` — Authored and verified 18 Antigravity skills in `.agents/skills/*/SKILL.md` and `.agents/skills/SKILLS_INDEX.md`.
  - `S0-P2-T1`: `PLANNED` -> `VERIFIED` — Implemented Rust core foundation crate (`crates/miniai-core`) with `#![forbid(unsafe_code)]`, 7 modality adapters, CPU-first hardware probe, 85% admission guard, SHA-256 verification, and `PrivacyPolicyEngine`.
  - `S0-P2-T2`: `PLANNED` -> `VERIFIED` — Executed `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test --workspace` (`7/7` tests passed, `0` warnings).
  - `S0-P2-T3`: `PLANNED` -> `VERIFIED` — Implemented Flutter cross-platform application (`ui/miniai_app`) with reactive `PlatformController` and accessible `DashboardScreen`.
  - `S0-P2-T4`: `PLANNED` -> `VERIFIED` — Executed `flutter analyze --no-pub` (`0` issues) and `flutter test --coverage` (`5/5` tests passed, `97.32%` measured LCOV line coverage: `218/224` lines).
  - `S0-P2-T5`: `PLANNED` -> `VERIFIED` — Implemented `ml/src/miniai_ml` (`contracts.py`, `runtime_probe.py`, `governance.py`) and `scripts/verify_stage0.py`.
  - `S0-P2-T6`: `PLANNED` -> `VERIFIED` — Executed `uv run pytest --cov=miniai_ml --cov-fail-under=90` (`6/6` tests passed, `98.16%` measured coverage: `213/217` statements).
  - `S0-P3-T1`: `PLANNED` -> `VERIFIED` — Created `Dockerfile`, `docker-compose.yml`, and `.devcontainer/devcontainer.json` with non-root user (`uid 10001`).
  - `S0-P3-T2`: `PLANNED` -> `VERIFIED` — Created multi-platform GitHub Actions workflows (`.github/workflows/ci.yml`, `security.yml`, `release.yml`) covering Windows, Linux, macOS, Android, and iOS.
  - `S0-P3-T3`: `PLANNED` -> `VERIFIED` — Created and linked remote GitHub repository `https://github.com/dayashimoga/miniAI.git`.
  - `S0-P3-T4`: `PLANNED` -> `VERIFIED` — Independent QA Lead (`independent-qa-auditor`) and Principal Architecture & Security Reviewer (`arch-security-reviewer`) both returned `PASS`.
- **Next Stage Ready:** `STAGE-01-BACKLOG` (Stage 1: Rust Core, Flutter Shell, SQLite Storage, IPC APIs & Typed Event System).

---

### [2026-11-05T12:28:00+05:30] Stage 1 — Entry 003 (Subphase 1.1.1 Completed & Verified; CLAUDE.md Adapted to Antigravity)
- **Entry Type:** `STATUS_TRANSITION`
- **Stage / Phase:** `Stage 1 / Phase 1.1 — Transactional SQLite Storage / Subphase 1.1.1`
- **Status Transitions:**
  - `ANTIGRAVITY-CLAUDE-INTEGRATION`: `COMPLETED` & `VERIFIED` — Authored `GEMINI.md` adapting `CLAUDE.md` behavioral tenets (Think Before Coding, Simplicity First, Surgical Changes, Goal-Driven Execution) to Antigravity's native environment, tool execution, artifacts, and specialist subagent workflows. Registered `GEMINI.md` in `ml/src/miniai_ml/governance.py`.
  - `S1-P1-SUB1.1.1` [`VERIFIED`]: Implemented `crates/miniai-storage` featuring:
    - Connection management with SQLite WAL mode (`PRAGMA journal_mode = WAL`), foreign keys (`PRAGMA foreign_keys = ON`), synchronous normal mode, and 5000ms busy timeout.
    - Deterministic schema migration engine (`MigrationRunner`) with SHA-256 checksum verification and `schema_migrations` tracking table.
    - Initial baseline schemas: `app_settings` key-value table, `workspaces` table with active switching, `consent_audit_log` immutable audit table, and `diagnostic_events` store.
    - Database integrity checks (`PRAGMA integrity_check` & `PRAGMA foreign_key_check`) reporting structured `IntegrityReport`.
    - Executed 10 unit and integration tests in `crates/miniai-storage` (17 tests total across workspace), achieving 100% pass rate with `#![forbid(unsafe_code)]`.
- **Next Subphase Ready:** `S1-P1-SUB1.1.2` (Subphase 1.1.2: Repository CRUD Contracts, Migration Rollback/Recovery & Crash-Recovery Integration Tests).

---

### [2026-11-05T14:11:00+05:30] Stage 1 — Entry 004 (Subphase 1.1.2 Completed & Verified)
- **Entry Type:** `STATUS_TRANSITION`
- **Stage / Phase:** `Stage 1 / Phase 1.1 — Transactional SQLite Storage / Subphase 1.1.2`
- **Status Transitions:**
  - `S1-P1-SUB1.1.2` [`VERIFIED`]: Implemented repository CRUD contracts, atomic transaction helpers, and corruption quarantine/recovery:
    - `CachedModelRecord` repository with insert/upsert, single get, list all, and modality-filtered queries.
    - `ChatSessionRecord` repository with insert/upsert, single get, list ordered by updated timestamp descending, and title update with timestamp bump.
    - `transaction<F, R>()` atomic helper guaranteeing complete rollback on closure error and commit on success.
    - `quarantine_and_recover()` helper: safely quarantines corrupt SQLite files to timestamped `.corrupt.bak` (plus WAL/SHM files) and re-creates clean, healthy database at the original path.
    - Verified crash-recovery, transaction rollback, and corruption quarantine integration tests (14 tests in `miniai-storage`, 21 tests total across Rust workspace).
- **Next Subphase Ready:** `S1-P2-SUB1.2.1` (Phase 1.2 / Subphase 1.2.1: Async Broadcast Event Bus (`SystemEvent` stream) with Backpressure Bounds in `crates/miniai-server`).

---

### [2026-11-05T14:54:00+05:30] Stage 1 — Entry 005 (Subphase 1.2.1 Completed & Verified)
- **Entry Type:** `STATUS_TRANSITION`
- **Stage / Phase:** `Stage 1 / Phase 1.2 — Typed Event Bus & Local IPC API / Subphase 1.2.1`
- **Status Transitions:**
  - `S1-P2-SUB1.2.1` [`VERIFIED`]: Implemented `crates/miniai-server` with async broadcast event bus:
    - Strongly typed `SystemEvent` enum (`HardwareChanged`, `PrivacyPolicyUpdated`, `AdapterHealthChanged`, `DiagnosticAlert`, `ModelDownloadProgress`, `InferenceTokenStream`, `WorkspaceChanged`) with canonical JSON wire contracts.
    - `EventEnvelope` containing monotonic sequence number, epoch timestamp, and typed payload.
    - `EventBus` featuring bounded backpressure broadcast queues (`tokio::sync::broadcast`) and in-memory replay ring buffer for newly connecting subscribers.
    - `EventSubscriber` with async `recv()` and non-blocking `try_recv()`, handling subscriber lag gracefully via `EventBusError::Lagged`.
    - Executed 5 unit and integration tests in `crates/miniai-server` (26 tests total across Rust workspace), achieving 100% pass rate with `#![forbid(unsafe_code)]`.
- **Next Subphase Ready:** `S1-P2-SUB1.2.2` (Subphase 1.2.2: Localhost Loopback API / FFI Bridge with Per-Session Authentication Nonce in `crates/miniai-server`).

---

### [2026-11-05T15:36:00+05:30] Stage 1 — Entry 006 (Subphase 1.2.2 Completed & Verified)
- **Entry Type:** `STATUS_TRANSITION`
- **Stage / Phase:** `Stage 1 / Phase 1.2 — Typed Event Bus & Local IPC API / Subphase 1.2.2`
- **Status Transitions:**
  - `S1-P2-SUB1.2.2` [`VERIFIED`]: Implemented in-process IPC bridge (`IpcBridge`) and localhost loopback HTTP/SSE server (`IpcServer`) in `crates/miniai-server`:
    - Session authentication nonce enforcement: all API endpoints (`/v1/system/snapshot`, `/v1/privacy/policy`, `/v1/events/recent`, `/v1/events/stream`) require valid `Authorization: Bearer <nonce>` header; unauthorized requests return HTTP 401.
    - Security-enforced localhost binding: strictly rejects non-loopback bindings (`0.0.0.0`) to prevent network exposure.
    - Full endpoint suite: `GET /v1/health` (liveness), `GET /v1/system/snapshot` (hardware + privacy + adapter snapshot), `GET /v1/privacy/policy`, `POST /v1/privacy/policy` (with persistent settings updates and event publishing), and `GET /v1/events/stream` (real-time Server-Sent Events).
    - Executed 8 unit and integration tests in `crates/miniai-server` (29 tests total across Rust workspace), achieving 100% pass rate with `#![forbid(unsafe_code)]`.
- **Next Subphase Ready:** `S1-P3-SUB1.3.1` (Phase 1.3 / Subphase 1.3.1: Persistent Navigation Rail/Drawer, Live Event Drawer & SQLite-Backed Settings Persistence in Flutter UI `ui/miniai_app`).


---

### [2026-11-05T18:25:00+05:30] Stage 1 — Entry 007 (Subphase 1.3.1 Completed & Verified)
- **Entry Type:** `STATUS_TRANSITION`
- **Stage / Phase:** `Stage 1 / Phase 1.3 — Flutter Shell Navigation, Settings & Event Stream Inspector / Subphase 1.3.1`
- **Status Transitions:**
  - `S1-P3-SUB1.3.1` [`VERIFIED`]: Implemented persistent multi-pane navigation shell, SQLite-backed settings persistence, and live event stream inspector in `ui/miniai_app`:
    - Responsive multi-layout shell (`ShellScreen`): adaptive desktop `NavigationRail` (>=720px width) and mobile `NavigationBar` (<720px width), with persistent destinations for `Modality Studio`, `Settings & Privacy`, and `System Event Stream`.
    - Persistent settings persistence (`SettingsStore`, `FileSettingsStore`, `InMemorySettingsStore`): loads and saves `AppSettings` (`localOnlyMode`, `forceCpuOnly`, `activeWorkspaceId`, `ipcServerUrl`, `sessionNonce`) across process restarts offline.
    - Full Settings panel (`SettingsScreen`): privacy governance toggles, cloud consent token status & revocation controls, force CPU-only baseline execution, active workspace switcher, and loopback IPC server/authentication nonce configuration with connection diagnostics.
    - Live Event Inspector & Slide-out End Drawer (`EventInspectorView`): real-time event streaming with sequence badges, semantic type icons, UTC timestamps, JSON payload inspection modal, type filter chips (`privacy_policy_updated`, `hardware_changed`, `workspace_switched`, `diagnostic_alert`, `adapter_health_changed`), and manual event trigger/clear controls.
    - Localhost loopback IPC client (`IpcClient`): authenticates with session bearer nonce, connects to `/v1/health`, `/v1/system/snapshot`, `/v1/events/recent`, and streams Server-Sent Events (`/v1/events/stream`).
    - Verified 15 tests across `test/widget_test.dart` and `test/shell_and_settings_test.dart` with 100% pass rate, zero Flutter analyze warnings, and 97.27% measured line coverage (713/733 lines).
- **Next Subphase Ready:** `S1-P3-SUB1.3.2` (Subphase 1.3.2: End-to-end Rust-to-Flutter integration contracts, offline disconnected resilience, and accessibility validation).
