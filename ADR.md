# MiniAI — Architectural Decision Records (`ADR.md`)

This document records binding architectural decisions for **MiniAI**, including context, considered alternatives, trade-offs, and consequences.

---

## ADR-001: Tri-Language Modular Architecture (Rust Core + Flutter UI + Python/PyTorch ML)
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** MiniAI requires memory-safe, low-overhead local systems orchestration, a responsive native UI across 5 operating systems (Windows, Linux, macOS, Android, iOS), and first-class access to the PyTorch training/distillation ecosystem.
- **Decision:**
  - Use **Rust** (`crates/`) for the core supervisor, model registry, hardware probing, cryptographic SHA-256 verification, privacy enforcement, and runtime orchestration.
  - Use **Flutter** (`ui/miniai_app/`) for the cross-platform desktop and mobile UI.
  - Use **Python/PyTorch** (`ml/`, managed via `uv`) for dataset pipelines, LoRA/QLoRA fine-tuning, distillation, and evaluation harnesses.
- **Consequences:** Strong compile-time safety in core services, single-codebase multi-OS UI, and zero compromise on ML training tooling. Cross-language contracts are enforced via canonical `snake_case` JSON schemas tested across all three languages.

---

## ADR-002: Independently Pluggable Modality Runtime Adapters
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** Multimodal platforms often fail when they attempt to force LLMs, diffusion models, speech recognizers, neural TTS codecs, and video pipelines into a single monolithic inference engine.
- **Decision:** Define a lightweight `ModalityAdapter` trait in `miniai-core`. Each modality (`Llm`, `Vision`, `ImageGeneration`, `SpeechAsr`, `SpeechTts`, `Music`, `Video`) registers its own adapter with independent dependency graphs, process isolation, and health checks.
- **Consequences:** Adding or upgrading a vision or audio model runtime never risks breaking LLM inference, and crashed adapters can be restarted independently.

---

## ADR-003: CPU-First Execution with Dynamic Hardware Fallback
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** Developer laptops, CI runners, and edge devices frequently lack discrete NVIDIA GPUs.
- **Decision:** Every modality adapter and ML workflow must implement and verify a deterministic `AcceleratorBackend::Cpu` execution path first. Hardware accelerators (`Cuda`, `Metal`, `Vulkan`, `DirectMl`, `Rocm`, `CoreMl`, `Nnapi`) are dynamically probed and selected with automatic fallback to `Cpu`.
- **Consequences:** 100% of MiniAI functional tests run deterministically on standard CPU hosts and CI runners while taking advantage of accelerators when present.

---

## ADR-004: Default Local-Only Privacy Mode & Cryptographic Cloud Consent Gates
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** Users of local-first AI platforms require an ironclad guarantee that prompts, documents, and media never leave their device without explicit permission.
- **Decision:** `PrivacyPolicyEngine` defaults to `local_only_mode = true`. Cloud or distributed compute requires both disabling `local_only_mode` and supplying a non-expired, endpoint- and modality-scoped `CloudConsentToken`.
- **Consequences:** Accidental data exfiltration is impossible by construction and verified by unit and security tests.

---

## ADR-005: Strictly Append-Only Engineering Logs (`TODO.md` and `CHANGELOG.md`)
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** In-place editing of task lists and changelogs destroys historical auditability across multi-stage engineering projects.
- **Decision:** `TODO.md` and `CHANGELOG.md` are strictly append-only. Every task creation, status transition, completion, and stage release is appended as a new timestamped entry at EOF and validated automatically by `scripts/verify_stage0.py`.
- **Consequences:** Full immutable audit trail of project execution from Stage 0 through Stage 10.



---

## ADR-006: SQLite WAL Mode, Deterministic Migrations & Localhost-Only Authenticated IPC
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** MiniAI requires robust local storage for workspace settings, model registry metadata, and audit logs, coupled with a responsive, secure communication channel between the Rust core supervisor and the Flutter UI shell.
- **Decision:**
  - Standardize on embedded SQLite using Write-Ahead Logging (`PRAGMA journal_mode = WAL`) and foreign key enforcement (`PRAGMA foreign_keys = ON`) in `crates/miniai-storage`.
  - Enforce deterministic database migrations verified by SHA-256 SQL checksums in `schema_migrations`, with atomic rollback and automated corruption quarantine recovery (`.corrupt.bak`).
  - Implement a typed asynchronous event bus (`tokio::sync::broadcast`) with bounded backpressure and late-subscriber replay ring buffers in `crates/miniai-server`.
  - Expose a localhost-only HTTP/SSE IPC server bound strictly to loopback (`127.0.0.1`), protected by per-session bearer nonce authentication (`Authorization: Bearer <nonce>`) on all endpoints.
- **Consequences:** Eliminates external network attack surfaces, prevents database locking under concurrent UI reads, ensures offline self-healing resilience, and provides real-time event streaming to the Flutter UI shell.


---

## ADR-007: Cryptographic Staging, Quarantine Isolation, Memory Admission Ceilings & Decoupled Modality Runtimes
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:** Managing multi-gigabyte neural network weights across 7 independent modalities requires rigorous protection against partial downloads, corrupted weights, unauthorized network exfiltration, system out-of-memory crashes, and cascading runtime failures.
- **Decision:**
  - Enforce isolated quarantine staging (`.part` for downloads, `.import.quarantine` for local file imports) in `crates/miniai-registry`. Streaming SHA-256 digests are computed during transfer, and files are atomically promoted to verified cache only upon exact digest validation. Corrupted or tampered files are purged immediately.
  - Reject remote downloads when `LocalOnlyPrivacyMode` is active while permitting fully offline local model imports.
  - Probe CPU SIMD instruction sets (`AVX2`, `AVX-512`, `NEON`) and host memory via `sysinfo` in `crates/miniai-core`. Enforce strict 85% available RAM and VRAM admission ceilings via `AdmissionGuard`.
  - Automatically fall back to `AcceleratorBackend::Cpu` when accelerator VRAM is insufficient, accelerators are absent, or `force_cpu_only` mode is set. Reject models with `MiniAiError::InsufficientMemory` if available system RAM is breached.
  - Orchestrate all 7 modalities via independent `ManagedAdapter` instances in `crates/miniai-runtime`, ensuring model switching or failure in one modality never degrades or crashes other active modalities.
- **Consequences:** Eliminates corruption risks in the model cache, guarantees system memory stability, prevents data leakage under local-only privacy mode, and ensures complete fault isolation across multimodal runtimes.
