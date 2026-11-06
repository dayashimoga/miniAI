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

---

## ADR-008: Local LLM Engine Architecture, Hot Model Switching with Two-Phase Drop, Conversation Tree Traversal & Quantitative Benchmarking
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:** Stage 3 requires production-grade offline LLM text generation, multi-session conversation persistence, and quantitative performance benchmarking on resource-constrained consumer hardware without cloud dependencies. Specific challenges include:
  1. Preventing Out-Of-Memory (OOM) spikes during model switching when two large models might temporarily coexist in memory.
  2. Supporting conversation branching, in-place message editing, and Markdown/JSON export without complex relational schemas.
  3. Achieving responsive token streaming (<10ms cancellation responsiveness) without thread blocking or memory leaks.
  4. Delivering quantitative benchmark metrics (cold start, TTFT, tokens/sec, memory RSS) with statistical integrity.
- **Decision:**
  - Implement a dedicated `#![forbid(unsafe_code)]` LLM crate (`crates/miniai-llm`) with an autoregressive decoder, GGUF magic verification (`0x46554747`), subword BPE tokenization, and a configurable sampler supporting greedy sampling, temperature scaling, nucleus top-p filtering, and exponential frequency repetition penalties.
  - Enforce a strict **Two-Phase Drop Pattern** for hot model switching: `LlmEngine::switch_model` calls `unload_model()` and asserts `self.weights.is_none()` before opening or mapping the replacement model file. This guarantees that peak memory during model transitions never exceeds the size of a single loaded model.
  - Implement SQLite Migration 4 (`004_chat_messages.sql`) utilizing a self-referential `parent_id` column on `chat_messages`. Linear conversation history is reconstructed via upward recursive traversal (`get_message_branch`), while alternative completions are indexed via child queries (`get_child_branches`), eliminating separate tree-link tables.
  - Integrate a responsive Flutter Chat Workspace (`ChatWorkspaceScreen`) with real-time token streaming badges, live tokens/second telemetry, and immediate cooperative generation cancellation (`cancel_generation_button`).
  - Implement `LlmBenchmarkHarness` in `crates/miniai-llm/src/benchmark.rs` to measure real cold load times from disk, warm memory reloads, and per-iteration TTFT and throughput. Enforce strict warmup iteration exclusion from reported p50, p95, and mean statistics, persisting output in conformance with `miniai-evaluation-benchmarks` JSON schema.
- **Consequences:** Eliminates memory exhaustion during model transitions, guarantees non-destructive conversation branching and editing, maintains offline-first privacy guarantees with zero cloud leakage, and provides mathematically honest performance benchmarks.


---

## ADR-009: Dataset Curation, Parameter-Efficient Fine-Tuning (LoRA), Hinton Knowledge Distillation, and Cryptographic Checkpoint Provenance
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 4 requires a robust dataset curation pipeline, local-first parameter-efficient fine-tuning (PEFT), teacher-student knowledge distillation, cryptographic model provenance tracking, and an interactive Flutter training studio. Key engineering challenges include:
  1. Ingestion of untrusted datasets with variable schemas (`prompt`/`response`, `instruction`/`output`, `input_text`/`target_text`) without malformed UTF-8 characters or path traversal.
  2. Executing fine-tuning on consumer CPU hardware where full parameter updating of large models is intractable.
  3. Ensuring identity initialization in LoRA adapters ($h = W_0 x$ at $t=0$) so un-tuned adapters do not degrade base model weights before training commences.
  4. Teacher-student distillation loss balancing task accuracy with temperature-scaled soft distributions preserving $T^2$ gradient scale.
  5. Cryptographic reproducibility: every produced adapter checkpoint must record exact base model SHA-256, dataset SHA-256, random seed, hyperparameters, and convergence metrics.
  6. Seamless export and integration of fine-tuned weights into the Stage 2 Model Registry without manual reconfiguration.
- **Decision:**
  - Build `miniai_ml` in Python (`ml/src/miniai_ml/`) managed via `uv`, with PyTorch CPU-first execution (`torch>=2.15.0+cpu`).
  - Implement `miniai_ml/dataset.py`: multi-format loader (`.jsonl`, `.csv`), UTF-8 strict decoding, flexible field normalization, content-based deduplication, and deterministic PRNG splitting with cryptographic SHA-256 manifest verification.
  - Implement `miniai_ml/training.py`:
    - `LoraLinear`: low-rank matrix decomposition ($W_0 + \frac{\alpha}{r} B A$), initialized with $A \sim \text{Kaiming uniform}$ and $B = 0$ (identity initialization). In-place weight merging (`merge`) and unmerging (`unmerge`) preserving exact numerical equivalence.
    - `DistillationLoss`: $\mathcal{L}_{total} = \alpha_{task} \mathcal{L}_{task} + (1 - \alpha_{task}) T^2 \mathcal{L}_{KD}$ over softened teacher logits with $T^2$ gradient scale preservation.
    - `Trainer`: forward/backward optimization loop with AdamW, gradient accumulation, gradient norm clipping, periodic validation evaluation, atomic checkpoint persistence/resumption, and cooperative cancellation callback.
  - Implement `miniai_ml/provenance.py`: cryptographic manifest `checkpoint_provenance.json` tracking base model SHA-256, dataset SHA-256, random seed, device info, hyperparameters, and convergence metrics. Export catalog manifests directly compatible with Stage 2 `ModelManifest`.
  - Implement `TrainingStudioScreen` in Flutter (`ui/miniai_app`) as Tab 6 in `ShellScreen`: dataset split inspector, LoRA tuning form, live progress bar, loss sparkline, and base vs tuned evaluation comparison table.
- **Consequences:**
  Enables consumer hardware fine-tuning with zero cloud compute leakage, provides mathematical verification of adapter merging/unmerging, enforces end-to-end cryptographic traceability of trained artifacts, and seamlessly connects training outputs to the MiniAI model registry.
