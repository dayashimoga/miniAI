# MiniAI — Global Engineering Contract

**Document ID:** `MINIAI-GEC-001`  
**Version:** `1.0.0`  
**Status:** `ENFORCED`  
**Effective Date:** `2026-11-05`

---

## 1. Purpose & Authority

This **Global Engineering Contract (`GLOBAL_ENGINEERING_CONTRACT.md`)** is the binding engineering constitution for the **MiniAI** platform. It governs all architecture, code implementation, testing, security, documentation, CI/CD pipelines, release certification, and AI agent workflows across Stages 0 through 10.

No stage, phase, subphase, or pull request may bypass the clauses defined in this contract.

---

## 2. Core Engineering Principles (Integrated with `CLAUDE.md`)

### 2.1 Think Before Coding
- Inspect existing files, state assumptions explicitly, and identify architectural trade-offs before writing or modifying code.
- Verify hardware capabilities, OS environment, and toolchain versions before executing build or runtime commands.
- Document significant architectural decisions in `ADR.md`.

### 2.2 Simplicity First
- Deliver the minimum complete, production-grade solution that satisfies the current stage's acceptance criteria.
- Do not introduce speculative abstractions, unused trait hierarchies, or premature micro-optimizations.

### 2.3 Surgical Changes
- Modify only the files required for the active task or subphase.
- Preserve existing working code, tests, comments, and documentation.
- Clean up any unused imports or dead symbols introduced by your own changes.

### 2.4 Goal-Driven Execution
- Every task must begin with measurable acceptance criteria and end with objective verification evidence (command outputs, exit codes, coverage reports, and audit logs).

---

## 3. Non-Negotiable Technical Mandates

### 3.1 Zero Mock / Zero Placeholder Policy
1. **No Fake Data**: Production code paths and UI views must never return fabricated responses, static canned completions, or simulated progress bars.
2. **No Unconnected UI**: Every button, toggle, status badge, and configuration control in the Flutter UI must be wired to a real Rust core service endpoint and state store.
3. **Truthful Capability Reporting**: When a model runtime or hardware backend is unavailable on the host machine, the system must report its exact availability status (`Available`, `Unavailable(Reason)`, `UnsupportedOnPlatform`) rather than simulating execution.

### 3.2 Independently Pluggable Multimodal Architecture
1. **Decoupled Modalities**: LLM text generation, vision understanding, image generation/editing, speech/ASR/TTS, music generation, and video understanding/generation must operate through independent runtime adapter contracts.
2. **No Forced Shared Runtime**: Never require an image diffusion model, an audio codec model, and a quantized GGUF/ONNX LLM to execute inside a single monolithic runtime process.
3. **Graceful Isolation**: Failure, crash, or resource exhaustion in one runtime adapter must not corrupt the Rust core supervisor, SQLite metadata store, or other active adapters.

### 3.3 Local-First, CPU-First, & Consent-Gated Hybrid Compute
1. **CPU-First Guarantee**: Every supported modality workflow must have a deterministic, verified CPU execution path so MiniAI works on machines without discrete GPUs.
2. **Optional Hardware Acceleration**: GPU/NPU backends (CUDA, Vulkan, Metal, DirectML, ROCm, CoreML, NNAPI) are dynamically detected and optionally selected with automatic fallback to CPU on failure.
3. **Strict Offline Mode (`LocalOnlyPrivacyMode`)**:
   - Enabled by default for inference and workspace operations.
   - When active, outbound network connections for telemetry, cloud inference, or distributed compute are cryptographically and programmatically blocked at the core network policy layer.
4. **Explicit Cloud Consent**:
   - Hybrid cloud GPU or distributed compute (Stage 9) requires an explicit, auditable user consent token specifying the target endpoint, data payload scope, retention policy, and expiration.

### 3.4 Append-Only Governance Logs (`TODO.md` and `CHANGELOG.md`)
1. `TODO.md` and `CHANGELOG.md` are **strictly append-only**.
2. Existing lines must **never** be deleted, overwritten, reordered, or edited in place (not even changing `[ ]` to `[x]` on an earlier line).
3. Every task registration, status change (`PLANNED` -> `IN_PROGRESS` -> `COMPLETED` / `VERIFIED`), defect discovery, or release milestone must be appended as a new chronologically dated section at the end of the file.
4. Automated verification (`scripts/verify_stage0.py`) checks git diffs and structural integrity of `TODO.md` and `CHANGELOG.md` on every gate check and CI run.

---

## 4. Quality, Testing, and Coverage Requirements

### 4.1 Required Test Tiers
Each stage must implement and execute all applicable test tiers:
- **Unit Tests**: Pure logic, data structures, parsers, validators, and state transitions.
- **Contract Tests**: Rust <-> Flutter API serialization schemas, Rust <-> Python ML IPC/CLI schemas, and runtime adapter trait compliance.
- **Integration Tests**: Multi-component workflows (storage + registry + runtime + event bus).
- **UI / Widget Tests**: Flutter component rendering, accessibility semantics, responsive layout, and user interaction flows.
- **Offline & Failure/Recovery Tests**: Network disconnection handling, corrupted artifact detection (SHA-256 mismatch), interrupted downloads, and process crash recovery.
- **Security Tests**: Path traversal prevention, secret redaction, network policy enforcement in `LocalOnlyPrivacyMode`, and dependency vulnerability scanning.
- **Performance & Resource Tests**: Startup latency, memory footprint bounds, and streaming latency measurement.

### 4.2 Coverage Thresholds
- **Critical Business Logic**: Minimum **90% measured line/branch coverage** across Rust core crates (`crates/miniai-core`) and Python ML core modules (`ml/src/miniai_ml`).
- **Required Release Tests**: **100% pass rate** across all active test suites before any stage gate can pass.
- **Honest Reporting**: Coverage and test results must be produced by real execution tools (`cargo test`, `pytest --cov`, `flutter test --coverage`) and clearly distinguish:
  - `PROVEN_ON_HOST`: Executed and verified on the current machine.
  - `VERIFIED_IN_CI`: Executed on GitHub Actions target OS runners.
  - `HARDWARE_DEPENDENT`: Requires specific accelerator hardware (e.g., discrete NVIDIA GPU or Apple Silicon Metal) not present on the current runner; fallback CPU path is verified.

---

## 5. Stage Gate Protocol

No stage or subphase is complete until it passes the **7-Step Stage Gate Loop**:
1. **Define Success Criteria**: Document measurable acceptance tests before coding.
2. **Implement**: Write clean, modular, documented code adhering to language rules.
3. **Integrate**: Wire components end-to-end without orphaned code or mock bridges.
4. **Execute & Test**: Run linters, formatters, unit tests, integration tests, and coverage tools.
5. **Audit Regressions & Security**: Run independent QA and security verification checks.
6. **Update Documentation & Append-Only Logs**: Update architecture/codebase docs and append new dated entries to `TODO.md` and `CHANGELOG.md`.
7. **Provide Evidence & Gate Report**: Produce a complete gate report detailing changed files, architecture decisions, executed commands, actual test results, coverage percentages, artifacts, limitations, and remaining TODOs.

