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

