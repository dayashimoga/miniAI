# MiniAI — Product & System Requirements Specification (`REQUIREMENTS.md`)

**Document ID:** `MINIAI-REQ-001`  
**Version:** `1.0.0`  
**Status:** `APPROVED`

---

## 1. Executive Summary

**MiniAI** is a production-grade, modular, local-first multimodal AI platform with hybrid local/cloud compute and independently pluggable model architectures. It runs across **Windows, Linux, macOS, Android, and iOS**, combining a high-assurance **Rust** core, a responsive **Flutter** cross-platform UI, and **Python/PyTorch** machine learning tooling.

---

## 2. Functional Requirements (`FR`)

### 2.1 Core Foundation & Governance (`FR-CORE`)
- **FR-CORE-01**: The platform shall enforce a strict boundary between Rust core services (`crates/`), Flutter UI (`ui/miniai_app/`), and Python ML tooling (`ml/`).
- **FR-CORE-02**: The platform shall maintain persistent local state (configurations, model manifests, chat sessions, benchmark logs, audit trails) in an embedded transactional database with schema migration support.
- **FR-CORE-03**: The platform shall provide a typed asynchronous event bus streaming system health, hardware changes, download progress, and inference tokens to the UI.

### 2.2 Independently Pluggable Multimodal Runtimes (`FR-MOD`)
- **FR-MOD-01**: Each modality—**LLM Text (`llm`)**, **Vision Understanding (`vision`)**, **Image Generation & Editing (`image_generation`)**, **Speech ASR (`speech_asr`)**, **Speech TTS (`speech_tts`)**, **Music Generation (`music`)**, and **Video Understanding/Generation (`video`)**—shall operate via an independent `ModalityAdapter` runtime contract.
- **FR-MOD-02**: Unrelated model architectures shall never be forced to share a single runtime engine or process space.
- **FR-MOD-03**: Every model artifact downloaded or imported into the local registry shall be cryptographically verified against a 64-character SHA-256 hex digest before execution.

### 2.3 Hardware Detection & CPU-First Execution (`FR-HW`)
- **FR-HW-01**: All modalities shall provide a deterministic **CPU-first** execution path that functions without a discrete GPU.
- **FR-HW-02**: The hardware discovery service shall dynamically detect CPU cores/SIMD features, available system RAM, and optional hardware accelerators (`CUDA`, `Vulkan`, `Metal`, `DirectML`, `ROCm`, `CoreML`, `NNAPI`).
- **FR-HW-03**: If an optional hardware accelerator fails initialization or exhausts device memory, the runtime supervisor shall automatically fall back to CPU execution and emit an actionable diagnostic event.

### 2.4 Local-First Privacy & Consent-Gated Hybrid Compute (`FR-PRIV`)
- **FR-PRIV-01**: **Local-Only Privacy Mode (`LocalOnlyPrivacyMode`)** shall be enabled by default. When enabled, all outbound network requests for inference, cloud GPU offload, or telemetry are blocked at the core policy engine.
- **FR-PRIV-02**: Hybrid cloud GPU or distributed compute (Stage 9) shall execute **only** when `LocalOnlyPrivacyMode` is explicitly disabled by the user AND a valid, non-expired, cryptographically signed/scoped `CloudConsentToken` is granted for that specific task.

### 2.5 Training, Fine-Tuning, Distillation & RAG (`FR-ML`)
- **FR-ML-01**: Users shall be able to curate, validate, and split local datasets with SHA-256 provenance tracking (Stage 4).
- **FR-ML-02**: The Python/PyTorch ML subsystem shall support parameter-efficient fine-tuning (LoRA/QLoRA), knowledge distillation (teacher -> student), and quantitative evaluation harnesses (Stage 4).
- **FR-ML-03**: The platform shall provide local Retrieval-Augmented Generation (RAG), document/code chunking, vector indexing, and sandboxed local tool execution (Stage 5).

---

## 3. Non-Functional Requirements (`NFR`)

| ID | Category | Requirement | Target / Metric |
| :--- | :--- | :--- | :--- |
| **NFR-01** | **Zero Mock Policy** | Zero fake data, hardcoded demos, or unconnected UI controls in any production path. | `0` violations |
| **NFR-02** | **Offline Resilience** | 100% of local inference, registry, RAG, and UI inspection workflows function with network unplugged. | Verified by offline test suite |
| **NFR-03** | **Test Coverage** | Measured test coverage across critical business logic (`crates/miniai-core`, `ml/src/miniai_ml`). | `>= 90%` line coverage |
| **NFR-04** | **Release Pass Rate** | Pass rate for required unit, contract, integration, UI, offline, and security tests at stage gates. | `100%` passing |
| **NFR-05** | **Cross-Platform CI** | Automated CI workflows verifying Windows, Linux, macOS, Android, and iOS targets on appropriate OS runners. | Verified in `.github/workflows/` |
| **NFR-06** | **Accessibility** | Semantic labels, keyboard navigation, and responsive layouts from mobile (`360px`) to desktop (`4K`). | Verified by widget semantics tests |
| **NFR-07** | **Auditability** | `TODO.md` and `CHANGELOG.md` maintained as strictly append-only chronological logs. | Verified by `verify_stage0.py` |

