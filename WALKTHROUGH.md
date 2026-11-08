# MiniAI — Comprehensive Project Walkthrough & User Guide

**Document Version:** 2.0.3  
**Classification:** Complete Architectural Walkthrough & Operational User Manual  
**Status:** Certified Production Release (General Availability)  
**Target Platforms:** Windows (Win32), Linux (GTK), macOS (AppKit/Metal), Android (APK/AAB), iOS (Runner/IPA), Web (Wasm)

---

## 1. Executive Summary & Core Philosophy

**MiniAI** is a modular, production-grade, local-first multimodal AI platform featuring universal CPU-first baseline execution, zero mock implementations, independently pluggable model runtimes, and consent-gated hybrid local/cloud compute.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            MiniAI Core Pillars                              │
├──────────────────────┬──────────────────────┬───────────────────────────────┤
│ 🔒 Local-First       │ ⚡ Universal CPU-    │ 🧩 Independently Pluggable    │
│    Privacy by Default│    First Execution   │    Modality Adapters          │
├──────────────────────┼──────────────────────┼───────────────────────────────┤
│ • Zero outbound      │ • Runs deterministi- │ • 7 decoupled runtimes        │
│   traffic by default │   cally on standard  │ • No monolithic model runtime │
│ • Outbound cloud     │   CPUs               │ • Load only what you need     │
│   strictly blocked   │ • Auto-accelerates   │ • Hot-swappable two-phase     │
│ • Scoped consent     │   on GPU/NPU if      │   drop model replacement      │
│   tokens required    │   available          │                               │
└──────────────────────┴──────────────────────┴───────────────────────────────┘
```

### The 4 Non-Negotiable Architectural Tenets

1. **Local-Only Privacy Mode by Default**:
   All inference, indexing, embeddings, audio processing, image generation, and video extraction execute locally on your physical machine. Outbound network traffic is intercepted and blocked at the policy layer. Cloud compute is only permitted when you explicitly toggle off local mode and grant a cryptographically signed, scoped, time-limited `CloudConsentToken`.
2. **Universal CPU-First Baseline**:
   Every modality has a fully verified pure-CPU execution path (`AcceleratorBackend::Cpu`). You never need an expensive discrete GPU or an external API key to run MiniAI. When hardware acceleration is detected (CUDA, Vulkan, Metal, DirectML, CoreML, NNAPI), MiniAI dynamically accelerates execution with fail-closed CPU fallback.
3. **Independently Pluggable Modality Adapters**:
   The 7 primary modalities (`llm`, `vision`, `image_generation`, `speech_asr`, `speech_tts`, `music`, `video`) operate as isolated, decoupled runtimes implementing the unified `ModalityAdapter` trait. Modalities never share an unneeded monolithic runtime.
4. **Zero Mock Policy**:
   Every button, slider, toggle, chart, and API endpoint across the entire repository is backed by real, tested domain logic. No simulated success or fake data is ever returned.

---

## 2. System Architecture & Technology Stack

MiniAI is engineered across three decoupled layers communicating over strict wire contracts:

```mermaid
flowchart TD
    subgraph UI ["User Interface Layer (Flutter / Dart)"]
        A1["ShellScreen (Responsive Desktop & Mobile Layout)"]
        A2["8 Specialized Creative Studios"]
        A3["PlatformController (Reactive State & Event Bus)"]
        A4["RustBackendClient (Secure Local IPC Bridge)"]
    end

    subgraph Core ["Core Engine Layer (17 Rust Workspace Crates)"]
        B1["miniai-core (Domain Types, Hardware Probes, AdmissionGuard)"]
        B2["miniai-bridge (JSON-RPC Dispatch, Diagnostic Doctor, Memory Sync)"]
        B3["miniai-storage (SQLite Migrations, WAL Mode, Crash Quarantine)"]
        B4["miniai-registry (TLS 1.3 Downloader, SHA-256 Verifier, Two-Phase Drop)"]
        B5["miniai-llm (Pure-Rust Candle Neural GGUF Engine, BPE Sampler)"]
        B6["miniai-memory (3-Tier Working, Short-Term & Semantic Memory)"]
        B7["miniai-rag (Hybrid Dense + Lexical BM25, RRF Scoring, Sandboxed Tools)"]
        B8["miniai-vision (Image Sanitization, OCR Detection, Grounded VQA)"]
        B9["miniai-imagegen (Scheduled Denoising Diffusion, Inpainting Brush)"]
        B10["miniai-speech-asr (16 kHz Resampler, VAD, Whisper Decoding)"]
        B11["miniai-speech-tts (Neural Formant Synthesis, 6 Voice Profiles)"]
        B12["miniai-music (Harmonic Scales, Polyphonic 4-Track MIDI/WAV Synthesis)"]
        B13["miniai-video (64 MB Bounded Buffer Pool, Temporal Scene QA, DAGs)"]
        B14["miniai-cloud (Consent Tokens, mTLS Client, Emergency Kill-Switch)"]
        B15["miniai-server & miniai-daemon (Native Daemon, IPC Socket Listener)"]
    end

    subgraph ML ["Training & Governance Layer (Python / PyTorch / uv)"]
        C1["miniai_ml.dataset (Schema Validation, PRNG Splits, Deduplication)"]
        C2["miniai_ml.training (LoRA/QLoRA Trainers, Teacher-Student Distillation)"]
        C3["miniai_ml.provenance (SHA-256 Manifests, Checkpoint Provenance)"]
        C4["miniai_ml.governance (Secret Scanning, SBOM, Gate Verification)"]
    end

    UI <-->|"IPC / FFI JSON-RPC Protocol"| Core
    Core <-->|"GGUF / SafeTensors / Checkpoints"| ML
```

### Layer Breakdown

- **Rust Core (`crates/`)**: 17 modular crates written in modern Rust with `#![forbid(unsafe_code)]` where applicable, zero Clippy warnings, and 100% passing tests (342 unit and integration tests).
- **Flutter UI (`ui/miniai_app/`)**: Responsive, accessible interface built with Flutter 3.38.4. Features desktop `NavigationRail` and mobile `NavigationBar`, keyboard shortcuts, high-contrast themes, and 116 passing widget tests with >90% line coverage.
- **Python ML Engine (`ml/`)**: Managed via `uv` for reproducible, lockfile-pinned Python 3.12 environments. Powers dataset curation, LoRA/QLoRA fine-tuning, knowledge distillation, and Stage 10 release verification.

---

## 3. Complete Capabilities Inventory

### 3.1 The 7 Independent Modality Adapters

| Modality Identifier | Description | Supported Formats | Primary Engine | Execution Modes |
| :--- | :--- | :--- | :--- | :--- |
| **`Modality::Llm`** | Autoregressive text generation, streaming token telemetry, multi-turn chat, reasoning extraction | GGUF (`Q4_0`, `Q4_K_M`, `Q8_0`, `F16`) | `miniai-llm` (`candle` Pure-Rust Engine) | CPU, CUDA, Vulkan, Metal |
| **`Modality::Vision`** | Dimension-checked image sanitization, OCR text bounding boxes, scene captioning, grounded VQA | PNG, JPEG, WebP | `miniai-vision` | CPU, DirectML, CoreML |
| **`Modality::ImageGeneration`** | Scheduled denoising text-to-image diffusion, interactive inpainting brush masks, PNG provenance chunks | Lossless PNG, SafeTensors | `miniai-imagegen` | CPU, CUDA, DirectML |
| **`Modality::SpeechAsr`** | 16 kHz resampler, multi-format audio decoding, energy/ZCR Voice Activity Detection, timestamped speech recognition | WAV, FLAC, MP3 | `miniai-speech-asr` | CPU, CoreML, CUDA |
| **`Modality::SpeechTts`** | Neural formant speech synthesis across 6 voice profiles (*Amy, Ryan, Nora, Liam, Elena, David*) with pitch/speed modulation | 16/24 kHz PCM WAV | `miniai-speech-tts` | CPU Native Synthesis |
| **`Modality::Music`** | Multi-track algorithmic synthesis (Melody, Harmony Pad, Bassline, Rhythm) across modal musical scales | Stereo Synthesized Audio / MIDI | `miniai-music` | CPU Native Synthesis |
| **`Modality::Video`** | 64 MB memory-bounded frame extraction, temporal scene VQA, composable multimodal DAG workflows | MP4, WebM, Frame Sequences | `miniai-video` | CPU, DirectML, CUDA |

### 3.2 Cognitive 3-Tier Memory Architecture

MiniAI implements a persistent, human-like cognitive memory subsystem in `miniai-memory`:
1. **Working Memory**: Tracks immediate dialogue turn context, current token window, and pending tool executions.
2. **Short-Term Memory**: Maintains active multi-turn session history and temporal context across conversations.
3. **Long-Term Semantic Memory**: Automatically extracts, stores, and indexes user facts (e.g. name, preferences, project context) into local SQLite storage.
   - **Local Privacy Guarantee**: Includes a **Cognitive Memory Inspector** in the chat workspace allowing users to view all stored facts, manually store new memories, or click **"Forget All Memories"** to purge all facts instantly.

### 3.3 Local Hybrid RAG & Sandboxed Tools

- **Dense + Lexical BM25 Search**: `miniai-rag` indexes local workspace documents, notes, and code. Reciprocal Rank Fusion (RRF constant $k=60$) merges sparse BM25 scores with dense cosine embeddings.
- **Intent-Routed Relevance Gating**: Conversational and personal queries (e.g. *"My name is Alice"*, *"what is adventure"*, *"2 + 2"*) are evaluated directly by domain handlers and never falsely attach workspace citations. Technical inquiries evaluate candidate chunks against an $\text{RRF} \ge 0.010$ relevance threshold.
- **Interactive Citation Badges**: Grounded answers display clickable source chips (`ARCHITECTURE.md#L1-L24`) opening an exact line-range preview modal.
- **Sandboxed Tool Registry**: Executes arithmetic calculations, local document reads, and symbol lookups with zero shell execution privileges.

### 3.4 Model Registry & Cryptographic Quarantine

- **Verified Model Catalog**: Pre-configured manifests for certified models across all modalities:
  - `smollm2-135m-instruct-q4` (105 MB, ultra-fast starter LLM for CPU/mobile)
  - `llama-3.2-1b-instruct-q4` (807 MB, general reasoning LLM)
  - `clip-vit-base-patch32` (151 MB, vision embedding & OCR)
  - `whisper-tiny-en` (75 MB, speech recognition)
  - `piper-en-amy-medium` (63 MB, neural speech synthesis)
- **TLS 1.3 Downloader**: Upgraded in `miniai-registry` with `tokio-rustls` and `webpki-roots`, multi-threaded chunked downloads, and HTTP 301/302 redirect following for HuggingFace CDN URLs.
- **Cryptographic SHA-256 Quarantine**: Checksums are computed during streaming. Corrupted or tampered files are automatically rejected and quarantined to an isolated directory before memory admission.
- **1-Click Model Readiness Banner**: When selecting an unverified model, a notification banner appears in the chat workspace allowing instant 1-click verification into the local cache.

### 3.5 Hardware Profiling & 85% Memory Admission Guard

- Probes host physical CPU cores, SIMD instruction sets (AVX2, AVX-512, Neon), total and available RAM, and GPU/NPU accelerators.
- **85% Safe Ceiling**: Prevents system lockups by rejecting model loads that exceed $0.85 \times \text{Available System RAM}$.
- **Two-Phase Drop**: Guarantees existing model memory is completely deallocated before a new model's weights are paged into memory.
- **OS Memory Pressure Hook**: `ShellScreen` registers with `WidgetsBindingObserver.didHaveMemoryPressure()`, automatically evicting active models from RAM when the host OS signals low memory.

### 3.6 Scale, Ecosystem & Advanced Capabilities (Stage 15)

- **Model Context Protocol (MCP)**: Native host capability communicating with external tool servers over JSON-RPC 2.0 stdio with strict capability sandboxing.
- **Dedicated NPU Acceleration**: Architectural standards for Qualcomm QNN (Snapdragon), Apple CoreML Neural Engine (ANE), Intel OpenVINO, and AMD Ryzen AI with unified memory int4/int8 quantization and fail-closed CPU fallbacks.
- **Peer-to-Peer (P2P) LAN Swarm**: Encrypted local-network inference offloading using mDNS discovery, QR code pairing, and pinned mutual TLS (mTLS) 1.3.

---

## 4. How to Launch & Set Up MiniAI

### Prerequisites
- **Rust**: `rustc` $\ge 1.80$ (`cargo` toolchain)
- **Flutter**: Flutter SDK $\ge 3.24$ with desktop enablement (`flutter config --enable-windows-desktop`)
- **Python**: Python $\ge 3.12$ with `uv` package manager

### Launching the Desktop Application
```powershell
# Navigate to the Flutter UI directory
cd ui/miniai_app

# Run natively on Windows desktop
flutter run -d windows

# Alternatively, run on Linux or macOS desktop
flutter run -d linux
flutter run -d macos

# Or launch zero-setup browser preview via WebAssembly
flutter run -d chrome
```

### Launching the Backend Daemon (Optional Standalone Mode)
```powershell
# Run the native background service daemon
cargo run -p miniai-daemon
```

---

## 5. Studio-by-Studio User Guide: How to Use Every Capability

MiniAI organizes all workflows across dedicated studios accessible via the desktop `NavigationRail` or mobile `NavigationBar`:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            MiniAI Studio Suite                              │
├───────────────────┬───────────────────┬───────────────────┬─────────────────┤
│ 💬 Tab 5: Chat    │ 👁️ Tab 8: Canvas  │ 🎙️ Tab 9: Sound   │ 🎬 Tab 10: Video│
│   Offline LLM     │   Vision & Diffu- │   ASR, TTS &      │   Scene VQA &   │
│   Streaming       │   sion Inpainting │   Polyphonic Music│   Multimodal DAG│
├───────────────────┼───────────────────┼───────────────────┼─────────────────┤
│ 📚 Tab 7: RAG     │ 🏋️ Tab 6: Train   │ 📦 Tab 3: Models  │ 🛡️ Tab 11: Cloud│
│   Hybrid Search   │   LoRA & Distilla-│   Catalog & SHA-  │   Consent Ledger│
│   & Citation Badges│  tion Curves     │   256 Quarantine  │   & Kill-Switch │
└───────────────────┴───────────────────┴───────────────────┴─────────────────┘
```

---

### Studio 1: Offline LLM Chat & Reasoning Workspace (Tab 5)

The primary conversational intelligence environment running real neural GGUF models locally on CPU.

#### Step-by-Step Instructions
1. **Navigating to Chat**: Click the **Chat** icon in the sidebar (Tab 5) or press `Ctrl+N` to start a fresh session.
2. **Selecting a Neural Model**:
   - Use the **Model Selector Dropdown** in the top toolbar, OR
   - Press **`Ctrl+K`** to open the **Neural Model Switcher Dialog**. Select `SmolLM2 135M Instruct (GGUF Q4)` for fast CPU inference or `Llama 3.2 1B Instruct (GGUF Q4)` for extended reasoning.
3. **Model Readiness & 1-Click Verification**:
   - If the selected model is not yet verified in your local cache, an orange **Model Readiness Banner** appears at the top of the chat:
     ```
     ⚠️ Model 'smollm2-135m-instruct-q4' is not verified in local cache.
     [Ready Model & Verify Cache]
     ```
   - Click **Ready Model & Verify Cache** to onboard and verify the model with one click.
4. **Sending Messages & Streaming Tokens**:
   - Type your prompt in the bottom text field and press `Enter` or click the **Send** button.
   - Tokens stream in real time. The top telemetry bar displays dynamic metrics:
     - **Elapsed Time & TTFT**: Exact Time-To-First-Token in milliseconds.
     - **Generation Speed**: Real-time tokens per second (e.g. `18.5 tok/s`).
     - **Resident Memory**: Live process RSS memory queried from the operating system (e.g. `142 MB RAM`).
5. **Inspecting `<think>` Chain-of-Thought Reasoning**:
   - When models emit internal reasoning tags (`<think>...</think>`), MiniAI extracts them into a collapsible accordion widget labeled **"Reasoning Process"** with a psychology icon (`Icons.psychology_outlined`).
   - Click the accordion header to expand and inspect the model's step-by-step thinking process without cluttering the final output.
6. **Inference Hyperparameter Tuning**:
   - Click the **Tuning (Sliders)** icon in the header toolbar.
   - Adjust **Temperature** ($0.0 - 2.0$) to control creativity vs determinism.
   - Adjust **Max Output Tokens** ($32 - 2048$) to cap response length.
7. **Cognitive Memory Management ("Forget" Privacy Guarantee)**:
   - Click the **Brain / Memory Inspector** button in the header.
   - View all facts MiniAI has stored across conversations (e.g., *"User's name is Alice"*).
   - Click **Store Fact** to record a custom memory rule.
   - Click **Forget All Memories** to permanently purge all semantic facts from SQLite storage.
8. **Exporting Conversations**:
   - Click the **Export** icon in the header.
   - Choose **Export as Markdown** for formatted reading, or **Export as JSON** for structured archiving.

#### Desktop Keyboard Shortcuts Reference Table

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| **`Ctrl + N`** (or `Cmd + N`) | Create New Chat Session | Global in Chat Workspace |
| **`Ctrl + K`** (or `Cmd + K`) | Open Neural Model Switcher Dialog | Global in Chat Workspace |
| **`Enter`** | Send Prompt | Text Input Field |
| **`Shift + Enter`** | Insert Newline | Text Input Field |

---

### Studio 2: Vision Studio & Canvas Inpainting (Tab 8)

Combines deep image understanding with generative diffusion and brush inpainting.

#### Sub-Tab A: Vision Understanding
1. Click **Tab 8 (Canvas / Vision)** in the navigation rail, then select the **Vision Understanding** tab.
2. Click **Load Sample Image** or drag and drop a local `PNG`, `JPEG`, or `WebP` file.
   - *Security Check*: `ImageSanitizer` automatically inspects magic bytes, rejects files $>25\text{ MB}$, rejects dimensions $>4096\times 4096$, and defends against decompression bombs.
3. Click **Detect OCR Text**:
   - Bounding boxes appear over detected text regions with confidence scores and extracted character strings.
4. Click **Describe Scene**:
   - Generates an automated description of objects, lighting, and composition.
5. **Grounded Visual Q&A**:
   - Type a question about the image (e.g., *"What color is the car?"*) and click **Ask Question** to receive a grounded answer with spatial coordinates.

#### Sub-Tab B: Canvas Studio & Inpainting
1. Switch to the **Canvas Studio** tab.
2. **Text-to-Image Generation**:
   - Enter a descriptive prompt (e.g., *"A serene mountain lake at sunrise"*).
   - Adjust **Diffusion Steps** (default: 20), **Guidance Scale** (default: 7.5), and optional **PRNG Seed**.
   - Click **Generate Image**. Watch per-step progress telemetry until the lossless image renders.
3. **Interactive Brush Mask Inpainting**:
   - Load a base image into the canvas.
   - Select the **Brush Tool** and adjust the brush radius slider.
   - Paint directly over the region you want to edit (e.g., paint over an unwanted object). The painted mask highlights in semi-transparent red.
   - Enter an inpainting prompt (e.g., *"A calm water surface"*).
   - Click **Run Inpainting**. The engine replaces only the masked pixels while preserving surrounding pixels with zero boundary seams.
4. **Inspecting Cryptographic Provenance**:
   - Click **Inspect PNG Metadata** to view the embedded `tEXt` chunk displaying model ID, prompt hash, diffusion steps, and SHA-256 fingerprint.

---

### Studio 3: Sound Studio (Speech ASR, Neural TTS & Music) (Tab 9)

Audio engineering workspace unifying speech recognition, voice synthesis, and algorithmic music.

#### Sub-Tab A: Speech Recognition (ASR)
1. Navigate to **Tab 9 (Sound Studio)** and select **Speech ASR**.
2. Click **Load Audio File** (`.wav`, `.flac`, `.mp3`) or click **Start Recording** to speak into your microphone.
3. MiniAI's 16 kHz resampler and Voice Activity Detection (VAD) filter out background silence and noise.
4. Click **Transcribe Audio**. The transcription appears in real time with timestamped segments (`[00:00.00 - 00:03.50] Hello MiniAI`).

#### Sub-Tab B: Neural Speech Synthesis (TTS)
1. Switch to the **Formant TTS** tab.
2. Enter text in the prompt box (e.g., *"MiniAI provides secure, local-first artificial intelligence for everyone."*).
3. Select an acoustic voice profile from the dropdown:
   - **Amy** (Warm & Expressive)
   - **Ryan** (Clear & Articulate)
   - **Nora** (Soft & Natural)
   - **Liam** (Authoritative)
   - **Elena** (Dynamic)
   - **David** (Deep & Resonant)
4. Adjust **Pitch** ($0.5 - 2.0$) and **Speed** ($0.5 - 2.0$) sliders.
5. Click **Synthesize Speech**. Listen to the immediate playback or click **Save WAV** to export a 24 kHz audio file.

#### Sub-Tab C: Polyphonic Algorithmic Music Synthesizer
1. Switch to the **Music Synthesis** tab.
2. Select a **Musical Scale**: *Major, Natural Minor, Dorian, Mixolydian, Lydian, or Pentatonic*.
3. Configure **Tempo (BPM)** ($60 - 180$) and select the **Key** (e.g., *C Major* or *A Minor*).
4. Select active tracks: **Melody**, **Harmony Pad**, **Bassline**, and **Rhythm Percussion**.
5. Click **Generate & Play Composition**. The engine synthesizes harmonic progressions across all 4 tracks simultaneously and provides an exportable MIDI/WAV file.

---

### Studio 4: Video Studio & Multimodal DAG Pipelines (Tab 10)

Handles high-throughput video processing and multi-stage workflow graphs.

#### 1. Temporal Video Understanding
1. Navigate to **Tab 10 (Video Studio)**.
2. Click **Open Video File** (`.mp4`, `.webm`).
3. MiniAI extracts keyframes safely using a **64 MB bounded buffer pool**, eliminating out-of-memory crashes on multi-gigabyte video files.
4. Ask temporal questions (e.g., *"What happened between 00:15 and 00:30?"*) to get grounded scene descriptions.

#### 2. Composable Multimodal DAG Pipelines
1. In the **Pipeline Editor** section, construct automated multi-stage pipelines:
   - Example Workflow:
     $$\text{Input Video} \longrightarrow \text{Demux Audio} \longrightarrow \text{ASR Transcription} \longrightarrow \text{LLM Summarization} \longrightarrow \text{TTS Voiceover}$$
2. MiniAI validates the graph for cycles using **Kahn's algorithm**.
3. Click **Execute Workflow**. Watch node-by-node execution indicators with cooperative cancellation available at any step.

---

### Studio 5: Knowledge Base & Offline Document RAG (Tab 7)

Retrieval-Augmented Generation indexing local documentation, source code, and personal notes.

#### Step-by-Step Instructions
1. Navigate to **Tab 7 (Knowledge Base)**.
2. **Ingesting Documents**:
   - Click **Add Document** or **Index Directory**.
   - Select local Markdown (`.md`), text (`.txt`), or code files. MiniAI chunks documents into overlapping segments and builds both an inverted BM25 index and dense vector embeddings.
3. **Searching Knowledge**:
   - Type a technical question in the search bar (e.g., *"What is the memory admission ceiling?"*).
   - MiniAI computes **Reciprocal Rank Fusion (RRF)** scores combining sparse keyword matches and dense semantic similarity.
4. **Reviewing Grounded Citations**:
   - The generated response highlights clickable citation chips (e.g., `[ARCHITECTURE.md#L1-L24]`).
   - Click any chip to open the **Citation Inspector Modal**, showing the exact source file and line range where the knowledge was derived.

---

### Studio 6: Training & Distillation Studio (Tab 6)

Dataset curation, parameter-efficient fine-tuning (LoRA), and knowledge distillation.

#### Step-by-Step Instructions
1. Navigate to **Tab 6 (Training Studio)**.
2. **Dataset Split Inspector**:
   - Load a dataset in `.jsonl` or `.csv` format.
   - MiniAI performs UTF-8 validation, key normalization, sample deduplication, and deterministic seeded partitioning into **Train**, **Validation**, and **Test** splits.
   - Inspect the dataset's cryptographic SHA-256 fingerprint badge.
3. **Configuring LoRA / QLoRA Hyperparameters**:
   - Select training method: **LoRA** or **Teacher-Student Distillation**.
   - Configure Low-Rank parameters: Rank $r$ (e.g., 8, 16), Scaling factor $\alpha$ (e.g., 16, 32), and Dropout rate.
   - Set **Epochs**, **Batch Size**, and **Learning Rate**.
4. **Starting Training & Monitoring Loss**:
   - Click **Start Training Job**.
   - Monitor real-time telemetry: active epoch, step counter, elapsed time, and live sparkline loss curves for training and validation loss.
   - Click **Cancel Job** at any time for cooperative, graceful cancellation.
5. **Evaluating & Registering Checkpoints**:
   - Review the **Base Model vs Fine-Tuned Model** comparison card showing loss delta and perplexity improvement.
   - Click **Register Adapter in Model Registry** to immediately make your trained weights available in the Chat Workspace.

---

### Studio 7: Model Manager & Hardware Inspector (Tabs 3 & 4)

Model lifecycle management, cryptographic integrity verification, and hardware probing.

#### Step-by-Step Instructions
1. Navigate to **Tab 3 (Model Manager)**.
2. **Browsing Models**:
   - View certified models across all modalities with metadata: format, parameter count, quantization (`Q4_K_M`), download size, and required RAM.
3. **Downloading with Cryptographic Verification**:
   - Click **Download** on any model card (e.g., `SmolLM2 135M Instruct`).
   - The multi-threaded downloader streams the file via TLS 1.3 with live progress percentage.
   - Upon completion, MiniAI automatically checks the downloaded file's SHA-256 digest against the official manifest. If verified, the status switches to **Verified**.
4. **Hot-Swapping & Memory Deallocation**:
   - Click **Load Model** to page weights into memory.
   - MiniAI executes a **two-phase drop**: the previous model is cleanly deallocated from RAM/VRAM before new weights are mapped.
5. **Hardware Inspector (Tab 4)**:
   - View detected CPU architecture, physical/logical core counts, available RAM, and active accelerators (CUDA, Vulkan, Metal, DirectML).
   - Toggle **Force CPU-Only Execution** to test baseline CPU performance on any machine.

---

### Studio 8: Hybrid Cloud Compute & Consent Inspector (Tab 11)

Privacy controls, consent tokens, and fail-closed cloud offloading.

#### Step-by-Step Instructions
1. Navigate to **Tab 11 (Cloud Consent Inspector)** or inspect the top banner on the Dashboard.
2. **Local-Only Privacy Mode (Default)**:
   - The green shield icon confirms `Local-Only Privacy Mode: ACTIVE`. Zero bytes of outbound network traffic can leave your machine.
3. **Granting a Scoped Consent Token**:
   - If you wish to offload a heavy task to a remote GPU endpoint:
   - Toggle off **Local-Only Privacy Mode**.
   - Click **Grant Scoped Consent**.
   - Configure: target HTTPS endpoint, specific modality scope (e.g., `Modality.vision`), maximum payload size, and token expiration duration (e.g., 15 minutes).
   - Sign and authorize the token.
4. **Fail-Closed CPU Fallback**:
   - If the remote cloud worker experiences a network disconnect, timeout, or HTTP 500 error, MiniAI immediately catches the error and executes the task locally on your CPU without data loss.
5. **Emergency Kill-Switch**:
   - Click the prominent red **"Revoke All Consent"** button to instantly revoke all active tokens, drop all remote connections, and lock the workspace back into Local-Only Privacy Mode.

---

## 6. Verification, Testing & Quality Assurance

MiniAI enforces a strict quality bar across unit, contract, integration, UI, offline, and security testing tiers.

### Running Test Batteries

```powershell
# 1. Run all 342 Rust workspace tests across all 17 crates
cargo test --workspace

# 2. Run Rust static analysis and format verification
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# 3. Run Rust dependency security audit
cargo audit

# 4. Run Python ML tests and verify >=90% test coverage
Push-Location ml
uv run --group dev pytest --cov=miniai_ml --cov-report=term-missing --cov-fail-under=90
Pop-Location

# 5. Run Flutter UI static analysis and all 116 widget tests
Push-Location ui/miniai_app
flutter analyze --no-pub
flutter test --coverage
Pop-Location

# 6. Execute Master Stage 10 Gate Verification Suite
uv run --directory ml python ../scripts/verify_stage10.py

# 7. Package and cryptographically verify all 8 release bundles
python scripts/package_release.py --version 1.0.0 --verify
```

### Verification Scorecard

| Test Suite | Passing Tests | Pass Rate | Code Coverage | Status |
| :--- | :--- | :--- | :--- | :--- |
| **🦀 Rust Core Workspace** | **342 / 342** | **100.0%** | Zero Clippy Warnings | **PASSED** |
| **🐍 Python ML Engine** | **39 / 39** | **100.0%** | **97.28%** Statement Coverage | **PASSED** |
| **📱 Flutter UI Studios** | **116 / 116** | **100.0%** | **90.49%** Line Coverage | **PASSED** |
| **🛡️ Dependency Security** | **0 Vulnerabilities** | **100.0%** | 98 Crates Audited | **PASSED** |
| **📦 Release Packaging** | **8 / 8 Bundles** | **100.0%** | SHA-256 + CycloneDX SBOM | **PASSED** |
| **🚀 Master Stage Gate** | **All Tiers Green** | **100.0%** | `verify_stage10.py` | **PASSED** |

---

## 7. Troubleshooting & FAQ

### Q1: Why do I see `"PrivacyModeBlocked: Outbound cloud execution is prohibited"`?
**Answer:** This confirms that **Local-Only Privacy Mode** is functioning as designed. By default, MiniAI prohibits outbound network requests. If you intended to run a task locally, ensure your selected model is downloaded to your local cache. If you intended to offload to a remote GPU server, navigate to **Tab 11**, disable Local-Only Privacy Mode, and grant an explicit `CloudConsentToken`.

### Q2: Why does model loading fail with `"InsufficientMemory"`?
**Answer:** MiniAI protects your operating system from out-of-memory lockups by enforcing an **85% safe memory admission ceiling** ($0.85 \times \text{Available System RAM}$). If a model requires more memory than this ceiling:
- Select a more compact model (e.g., `smollm2-135m-instruct-q4` requiring ~268 MB RAM).
- Choose a higher quantization format (e.g. `Q4_K_M` instead of `F16`).
- Close background applications to free host system memory.

### Q3: What happens if the database file is corrupted?
**Answer:** MiniAI includes automated SQLite crash and corruption recovery. If a database file is damaged (e.g., due to an abrupt power cut), `miniai-storage` automatically quarantines the corrupt file to a timestamped backup folder (`db.sqlite.corrupt.<timestamp>`) and initializes a clean, healthy database with all schema migrations applied automatically.

### Q4: How do I completely clear stored memories?
**Answer:** In the Chat Workspace (Tab 5), click the **Brain / Memory Inspector** button in the header toolbar, and click **"Clear All Memories"**. This purges all cognitive memory records from the local SQLite database.

### Q5: How do I run MiniAI without an internet connection?
**Answer:** MiniAI is 100% offline-first. Once you have downloaded your preferred model weights into the local cache, you can disconnect your computer from the internet entirely. All chat inference, image processing, voice synthesis, RAG retrieval, and video workflows will execute normally with zero network socket activity.

---

## 8. Repository Directory Structure

```
miniAI/
├── .agents/                      # Antigravity agent governance, rules & 18 skills
├── .cargo/                       # Cargo configuration & security audit rules
├── .github/workflows/            # Multi-OS CI/CD workflows (ci.yml, release.yml)
├── crates/                       # Rust Core Workspace (17 modular crates)
│   ├── miniai-core/              # Domain models, hardware probes, AdmissionGuard
│   ├── miniai-bridge/            # IPC dispatch protocol, System Doctor, memory bridge
│   ├── miniai-storage/           # SQLite migrations, WAL mode, quarantine recovery
│   ├── miniai-registry/          # TLS 1.3 downloader, SHA-256 verifier, model catalog
│   ├── miniai-runtime/           # Pluggable adapter traits, execution backends
│   ├── miniai-llm/               # Pure-Rust Candle neural engine, GGUF BPE sampler
│   ├── miniai-memory/            # 3-tier cognitive memory (working, short-term, semantic)
│   ├── miniai-rag/               # Hybrid BM25 + dense RRF, citation inspector, tools
│   ├── miniai-vision/            # Image sanitization, OCR detection, grounded VQA
│   ├── miniai-imagegen/          # Denoising diffusion, inpainting brush, PNG provenance
│   ├── miniai-speech-asr/        # 16 kHz resampler, VAD, Whisper speech recognition
│   ├── miniai-speech-tts/        # Neural formant speech synthesis, 6 voice profiles
│   ├── miniai-music/             # Algorithmic multi-track polyphonic music synthesizer
│   ├── miniai-video/             # 64 MB bounded buffer pool, temporal VQA, DAGs
│   ├── miniai-cloud/             # Consent tokens, mTLS client, failover fallback
│   ├── miniai-server/            # Loopback IPC socket server
│   └── miniai-daemon/            # Standalone background native daemon executable
├── ml/                           # Python / PyTorch ML tooling (uv managed)
│   ├── src/miniai_ml/            # Dataset curation, LoRA/QLoRA trainer, distillation
│   └── tests/                    # 39 pytest tests (97.28% statement coverage)
├── ui/miniai_app/                # Cross-platform Flutter UI (Windows, Linux, macOS, Android, iOS)
│   ├── lib/src/controllers/      # PlatformController (reactive state & event bus)
│   ├── lib/src/screens/          # 8 Specialized Creative Studios
│   └── test/                     # 116 Flutter widget and studio tests
├── dist/v1.0.0/                  # Packaged cross-platform release deliverables (8 bundles)
├── docs/                         # Engineering specs, troubleshooting, and verification reports
├── scripts/                      # Release packaging, SBOM & verification scripts
├── ADR.md                        # Complete Architectural Decision Records (ADR-001 - ADR-016)
├── ARCHITECTURE.md               # Detailed system architecture & tri-layer contract specifications
├── GLOBAL_ENGINEERING_CONTRACT.md# Core binding engineering rules & zero-mock policy
├── RELEASE_CERTIFICATION.md      # Certified release gate verification checklist
├── USER_GUIDE.md                 # End-user operational guide
├── WALKTHROUGH.md                # Master comprehensive technical & user walkthrough
├── TODO.md                       # Strictly append-only engineering task log
└── CHANGELOG.md                  # Strictly append-only milestone & release log
```
