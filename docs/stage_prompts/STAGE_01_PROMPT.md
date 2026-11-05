# Stage 1 Technical Execution Prompt: Rust Core, Flutter Shell, Storage, APIs & Event System

## 1. Objective & Dependencies
- **Objective:** Build the transactional SQLite storage engine (`crates/miniai-storage`), typed asynchronous event bus and local IPC API (`crates/miniai-server`), and persistent multi-pane Flutter shell (`ui/miniai_app`).
- **Dependencies:** Stage 0 Gate Verified.
- **Active Skills:** `miniai-architecture`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-testing-qa`, `miniai-documentation-logs`, `miniai-staged-implementation`, `miniai-independent-audit`.

## 2. Phases & Subphases
- **Phase 1.1: Transactional SQLite Storage (`crates/miniai-storage`)**
  - *Subphase 1.1.1:* Connection pool with WAL mode, foreign keys, schema migration table, and workspace settings/audit log persistence.
  - *Subphase 1.1.2:* Atomic transaction helpers, database integrity check (`PRAGMA integrity_check`), and corruption recovery handler.
- **Phase 1.2: Typed Event Bus & Local IPC API (`crates/miniai-server`)**
  - *Subphase 1.2.1:* Bounded broadcast channel for `SystemEvent` (`HardwareChanged`, `PrivacyPolicyUpdated`, `AdapterHealthChanged`, `DiagnosticAlert`).
  - *Subphase 1.2.2:* Localhost IPC API endpoints (`/v1/system/snapshot`, `/v1/privacy/policy`, `/v1/events/stream`) protected by session bearer nonce.
- **Phase 1.3: Flutter Adaptive Shell & Live Event Inspector**
  - *Subphase 1.3.1:* Persistent navigation shell, SQLite-backed settings panel, and real-time event log drawer.
  - *Subphase 1.3.2:* Full Rust <-> Flutter integration and widget accessibility tests.

## 3. Security & Edge Cases
- Reject unauthenticated IPC requests lacking the per-process session nonce.
- Bind IPC listener strictly to `127.0.0.1` (loopback only).
- Handle locked or corrupted SQLite database files gracefully with diagnostic remediation.

## 4. Acceptance Tests & Exit Criteria
- `cargo test --workspace` and `flutter test` pass 100%.
- >= 90% measured line coverage on `miniai-storage` and `miniai-server`.
- Settings and privacy state persist across full process restarts offline.

