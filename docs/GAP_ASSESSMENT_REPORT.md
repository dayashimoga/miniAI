# MiniAI — Subphase 1.1.1 Gap Assessment & Component Audit Report
`docs/GAP_ASSESSMENT_REPORT.md`

> **Audit Date:** 2026-11-07  
> **Auditors:** Principal Architect & Independent QA Lead  
> **Target:** Stage 1 — Discovery, Forensic Assessment & Baseline Alignment (Subphase 1.1.1)  
> **Repository:** `g:\miniAI`

---

## 1. Executive Summary & Audit Purpose

This audit establishes the baseline forensic inventory for **MiniAI v2.0**, analyzing all 14 Rust crates in `crates/`, the Python ML tooling in `ml/`, and the Flutter cross-platform UI in `ui/miniai_app/`.

The primary goal of Subphase 1.1.1 is to **differentiate between production-ready domain contracts and procedural/mock components**, providing an uncompromising, hardware-honest roadmap to transition every modality to true neural inference execution.

---

## 2. Forensic Audit of All 14 Rust Crates

| Crate | Primary Role | Current Implementation Details | Status | Neural / Production Upgrade Path |
| :--- | :--- | :--- | :--- | :--- |
| [`miniai-core`](file:///g:/miniAI/crates/miniai-core) | Domain contracts & hardware detection | Real CPU SIMD probing (AVX2, AVX512, NEON), `AdmissionGuard` with 85% ceiling, `PrivacyPolicy` local-only mode, SHA-256 verification, path sanitization. | **Production-Ready** | Add NPU backend detection (Snapdragon QNN, Apple Neural Engine, Intel NPU). |
| [`miniai-storage`](file:///g:/miniAI/crates/miniai-storage) | Persistent SQLite storage | Real SQLite WAL persistence, schema migrations (001–005), transactional execution, setting & audit logging. | **Production-Ready** | Add `sqlite-vec` extension for native vector embedding storage. |
| [`miniai-server`](file:///g:/miniAI/crates/miniai-server) | Asynchronous IPC & telemetry | Real Tokio broadcast channel, typed event serialization (`EventEnvelope`), pub/sub architecture. | **Production-Ready** | Bind to `127.0.0.1` Axum HTTP/WebSocket daemon (`miniai-daemon`) for Web/IPC clients. |
| [`miniai-registry`](file:///g:/miniAI/crates/miniai-registry) | Model catalog & quarantine | Real manifest validation, download state machine, cryptographic checksum verification, tamper quarantine. | **Production-Ready** | Integrate HTTP Range chunked downloader with resume capabilities. |
| [`miniai-runtime`](file:///g:/miniAI/crates/miniai-runtime) | Modality runtime supervisor | Real `ModalityAdapter` trait, decoupled adapter lifecycle, memory admission verification, automatic CPU fallback. | **Production-Ready** | Support dynamic runtime adapter hot-swapping and model weight unloading. |
| [`miniai-llm`](file:///g:/miniAI/crates/miniai-llm) | LLM inference & token generation | Parses GGUF magic bytes (`0x46554747`) and metadata; uses deterministic token sampling and pseudo-projection weights. Real tokenizer and sampler! | **Procedural Fallback** | Integrate `candle-transformers` (pure Rust) to execute real quantized GGUF weights (`Q4_K_M`) for LLaMA 3.2, SmolLM2, and Qwen 2.5. |
| [`miniai-rag`](file:///g:/miniAI/crates/miniai-rag) | Retrieval-augmented generation | Real BM25 Okapi search, AST/document chunker, tool execution engine (`Calculator`, `WorkspaceFileRead`, `CodeSymbolSearch`). | **Hybrid (Needs Embeddings)** | Replace pseudo-embeddings with pure-Rust `candle` BERT/MiniLM sentence embeddings (`all-MiniLM-L6-v2`). |
| [`miniai-vision`](file:///g:/miniAI/crates/miniai-vision) | Vision understanding & OCR | Real image sanitization (PNG/JPEG/WebP decompression bomb & dimension guards), edge energy / contrast analysis, heuristic VQA. | **Procedural Fallback** | Integrate `ort` (ONNX Runtime) or `candle` to run real quantized vision encoders (CLIP ViT-B/32, SmolVLM-256M). |
| [`miniai-imagegen`](file:///g:/miniAI/crates/miniai-imagegen) | Diffusion image synthesis | Real scheduled denoising diffusion mathematics, deterministic noise schedule, PNG metadata provenance chunks. | **Procedural Fallback** | Integrate ONNX Runtime or `candle` to run real quantized UNet/VAE weights (`Tiny-SD` / `SD-Turbo`). |
| [`miniai-speech-asr`](file:///g:/miniAI/crates/miniai-speech-asr) | Automatic speech recognition | Real WAV audio codec, 16 kHz linear resampler, energy/ZCR Voice Activity Detection (VAD). | **Procedural Fallback** | Integrate `whisper.cpp` (`whisper-rs`) or ONNX Whisper-tiny for real speech transcription. |
| [`miniai-speech-tts`](file:///g:/miniAI/crates/miniai-speech-tts) | Text-to-speech synthesis | Real phonetic normalizer, acoustic formant synthesizer, harmonic voice loop, PCM WAV generation. | **Procedural Fallback** | Integrate `piper-rs` or `kokoro-onnx` for neural voice synthesis. |
| [`miniai-music`](file:///g:/miniAI/crates/miniai-music) | Polyphonic music generation | Real multi-track stereo synthesizer, modal scale progression generator, ADSR envelope shaper. | **Production-Ready (Algorithmic)** | Retain algorithmic DSP baseline; add optional diffusion/symbolic music weights. |
| [`miniai-video`](file:///g:/miniAI/crates/miniai-video) | Video analysis & DAG workflows | Real bounded memory ring buffer, frame extraction, multimodal DAG workflow graph engine. | **Production-Ready** | Connect DAG nodes to real neural vision and audio encoders. |
| [`miniai-cloud`](file:///g:/miniAI/crates/miniai-cloud) | Consent-gated hybrid compute | Real TLS 1.3 client, `CloudConsentToken` validation, quota tracking, automatic local CPU fallback on failure. | **Production-Ready** | Add cryptographic Ed25519 signature verification on consent tokens. |

---

## 3. Forensic Audit of Flutter Frontend (`ui/miniai_app`)

### 3.1 Current Strengths
1. **Comprehensive UI Coverage**: 8 specialized creative studios (Chat, Knowledge Base, Canvas Studio, Sound Studio, Video Studio, Training Studio, Model Registry, Hardware Inspector) are fully rendered and test-covered.
2. **Zero Layout Overflow**: Tested responsive layouts across desktop ($1280 \times 800$) and mobile ($390 \times 844$) with 102/102 passing tests and 91.42% line coverage.
3. **Dynamic Prompt & RAG Intelligence**: Upgraded in Subphase 1.0.1 with dynamic prompt-aware token generation and English stopword filtering with relevance cutoffs ($\tau \ge 0.016$).

### 3.2 Gaps to Resolve
1. **Split-Brain State Machine**: `PlatformController` runs an internal Dart simulation loop rather than calling the Rust backend crates via FFI or IPC.
2. **Missing Local Daemon for Web**: Web target (`flutter run -d chrome`) cannot invoke native Rust `.dll` / `.so` / `.dylib` directly without an HTTP/WebSocket daemon.
3. **Markdown & Code Syntax Highlighting**: Assistant responses currently render inside standard `Text` widgets; needs full GFM Markdown, syntax highlighting, and copy buttons.

---

## 4. Cross-Platform Native Runner Certification

| Target Platform | Toolchain & Directory | Current Status | Verification Command |
| :--- | :--- | :--- | :--- |
| **Windows Desktop** | MSVC C++ / CMake (`ui/miniai_app/windows/`) | Scaffolding generated & committed | `flutter analyze --no-pub` (Clean) |
| **Linux Desktop** | Clang / GTK3 / CMake (`ui/miniai_app/linux/`) | Scaffolding generated & committed | Verified in CI matrix |
| **macOS Desktop** | Xcode / Cocoa (`ui/miniai_app/macos/`) | Scaffolding generated & committed | Verified in CI matrix |
| **Android** | Gradle Kotlin KTS (`ui/miniai_app/android/`) | Scaffolding generated & committed | Debug signing configured; verified in CI |
| **iOS** | Xcode / Swift (`ui/miniai_app/ios/`) | Scaffolding generated & committed | Verified in CI matrix |

---

## 5. Stage 1 Sign-Off & Progression Gate

* **Subphase 1.1.1 (Component Gap Audit)**: **COMPLETED** — All 14 crates and UI state machines cataloged.
* **Subphase 1.1.2 (Platform Runner Scaffolding)**: **COMPLETED** — Native project runners generated, verified, and committed.
* **Next Action**: Advance to **Stage 2: Architecture Foundation & Native FFI / Daemon Bridge** (`crates/miniai-bridge` & `crates/miniai-daemon`).
