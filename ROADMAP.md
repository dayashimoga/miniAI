# MiniAI — Master Product & Engineering Roadmap (`ROADMAP.md`)

**Document ID:** `MINIAI-ROADMAP-001`  
**Version:** `1.0.0`

---

## 1. Roadmap Overview

MiniAI is engineered across **11 sequential stages (`Stage 0` through `Stage 10`)**. Each stage delivers a complete, integrated, tested, documented, and independently verified vertical slice of the platform. No stage may begin until the preceding stage has passed its Stage Gate Audit.

```mermaid
flowchart LR
    S0["Stage 0: Foundation, Governance & CI"] --> S1["Stage 1: Rust Core, Flutter Shell & Event Bus"]
    S1 --> S2["Stage 2: Model Registry, SHA-256 & Runtimes"]
    S2 --> S3["Stage 3: Offline LLM Chat, Streaming & Benchmarks"]
    S3 --> S4["Stage 4: Datasets, Fine-Tuning & Distillation"]
    S4 --> S5["Stage 5: Local RAG, Indexing & Secure Tools"]
    S5 --> S6["Stage 6: Vision & Image Gen/Editing"]
    S6 --> S7["Stage 7: Speech ASR, TTS, Voice & Music"]
    S7 --> S8["Stage 8: Video Understanding & Generation"]
    S8 --> S9["Stage 9: Consent-Gated Hybrid Cloud Compute"]
    S9 --> S10["Stage 10: Hardening, Certification & Releases"]
```

---

## 2. Stage Deliverables & Gate Matrix

| Stage | Name | Primary Deliverables | Core Verification Gate | Current Status |
| :--- | :--- | :--- | :--- | :--- |
| **Stage 0** | Repository, Governance, Architecture, Skills, Rules, CI & Foundation | Governance docs, 18 Antigravity skills, 6 rules, GitHub repo + CI workflows, Rust/Flutter/Python foundation packages, stage prompts (`00`–`10`), `verify_stage0.py` | 100% tests pass, >=90% coverage on core logic, independent audit `PASS` | `IN_PROGRESS` |
| **Stage 1** | Rust Core, Flutter Shell, Storage, APIs & Event System | SQLite transactional store, IPC/API server, typed event bus, Flutter adaptive desktop/mobile shell & settings | Storage CRUD + crash recovery tests, IPC contract tests, Flutter widget tests | `PLANNED` |
| **Stage 2** | Model Registry, Downloads, Integrity, Hardware Detection & Runtimes | Resumable model downloader, SHA-256 verifier, hardware detector, pluggable runtime supervisor | Corrupt weight rejection tests, CPU fallback tests, registry UI integration tests | `PLANNED` |
| **Stage 3** | Offline LLM Inference, Chat, Streaming, Switching & Benchmarking | Local CPU/GPU LLM runtime adapter, streaming token chat UI, hot model switching, latency/memory benchmark suite | Streaming chat E2E test, offline enforcement test, benchmark report validation | `PLANNED` |
| **Stage 4** | Dataset Management, Fine-Tuning, Training, Distillation & Evaluation | Dataset curator/splitter, LoRA/QLoRA trainer, teacher-student distillation, provenance manifests, evaluation dashboard | Deterministic CPU micro-training test, checkpoint provenance verification | `PLANNED` |
| **Stage 5** | RAG, Document/Code Indexing & Secure Tool Integration | Document/code AST chunker, local vector index, hybrid retrieval, sandboxed local tool runner | Citation accuracy test, sandbox escape prevention test, offline RAG E2E test | `PLANNED` |
| **Stage 6** | Vision, Image Understanding, Generation & Editing | Vision QA/captioning adapter, local image generation & inpainting adapter, multimodal canvas UI | Image format/bounds security tests, CPU vision/diffusion E2E tests | `PLANNED` |
| **Stage 7** | Speech, Audio, TTS, ASR, Voice & Music | Offline Whisper ASR adapter, neural TTS adapter, voice activity detection, music generation adapter & audio studio UI | WAV/FLAC pipeline tests, ASR transcript accuracy test, TTS audio output verification | `PLANNED` |
| **Stage 8** | Video Understanding, Generation & Media Workflows | Frame extraction & temporal vision analysis, local video synthesis adapter, timeline media workflow UI | Frame budget memory guard tests, multimodal media workflow integration tests | `PLANNED` |
| **Stage 9** | Hybrid Cloud GPU & Secure Distributed Compute | Explicit `CloudConsentToken` UI/engine, mTLS encrypted remote worker dispatch, automatic local fallback | Strict block when `LocalOnlyPrivacyMode=true`, consent revocation & expiry tests | `PLANNED` |
| **Stage 10** | Cross-Platform Hardening, Certification, Packaging & Releases | Full installers/bundles for Windows, Linux, macOS, Android, iOS, SBOM, SHA-256 release manifests, final certification audit | 100% release suite green across OS runners, zero open critical findings | `PLANNED` |

