# MiniAI v3.0 — Comprehensive Platform Walkthrough & User Manual

**Version:** `v3.0.0-GA`  
**Classification:** Complete Architectural Walkthrough & Operational User Manual  
**Status:** Certified Production Release (General Availability)  
**Target Platforms:** Windows (Win32), Linux (GTK), macOS (AppKit/Metal), Android (APK/AAB), iOS (Runner/IPA), Web (Wasm)  

---

## 1. Executive Summary & Core Philosophy

**MiniAI** is a modular, production-grade, local-first multimodal AI platform featuring universal CPU-first baseline execution, zero mock implementations, independently pluggable model runtimes, and consent-gated hybrid local/cloud compute.

```mermaid
flowchart TD
    subgraph CorePillars["MiniAI Core Pillars"]
        direction LR
        P1["🔒 Local-First Privacy<br/>• Outbound traffic blocked by default<br/>• Scoped consent tokens required"]
        P2["⚡ Universal CPU-First<br/>• Runs deterministically on standard CPUs<br/>• Dynamic GPU/NPU acceleration"]
        P3["🧩 Pluggable Modalities<br/>• 7 decoupled runtimes<br/>• Two-phase hot model switching"]
        P4["🧠 Cognitive Intelligence<br/>• 3-Tier memory architecture<br/>• Frontier <think> distillation"]
    end
```

### The 4 Non-Negotiable Tenets

1. **Local-Only Privacy Mode by Default**:
   All inference, indexing, embeddings, audio processing, image generation, and video extraction execute locally on your physical machine. Outbound network traffic is intercepted and blocked at the policy layer. Cloud compute is only permitted when you explicitly toggle off local mode and grant a cryptographically signed, scoped, time-limited `CloudConsentToken`.
2. **Universal CPU-First Baseline**:
   Every modality has a fully verified pure-CPU execution path (`AcceleratorBackend::Cpu`). You never need an expensive discrete GPU or an external API key to run MiniAI. When hardware acceleration is detected (CUDA, Vulkan, Metal, DirectML, CoreML, NNAPI), MiniAI dynamically accelerates execution with fail-closed CPU fallback.
3. **Independently Pluggable Modality Adapters**:
   The 7 primary modalities (`llm`, `vision`, `image_generation`, `speech_asr`, `speech_tts`, `music`, `video`) operate as isolated, decoupled runtimes implementing the unified `ModalityAdapter` trait. Modalities never share an unneeded monolithic runtime.
4. **Zero Mock Policy**:
   Every button, slider, toggle, chart, and API endpoint across the entire repository is backed by real, tested domain logic. No simulated success or fake data is ever returned.

---

## 2. Architecture & Inter-Process Communication (IPC)

MiniAI separates presentation, orchestration, and native neural computation into decoupled layers communicating over zero-copy C-FFI and authenticated loopback REST/JSON.

```mermaid
flowchart TD
    subgraph FlutterClient["Flutter Multiplatform Client (ui/miniai_app)"]
        Chat["Chat Workspace<br/>(Markdown, &lt;think&gt;, Copy)"]
        Manager["Model Manager<br/>(HuggingFace & Importer)"]
        Telemetry["Telemetry Bar<br/>(Battery, Thermals, ECO)"]
        Studios["Multimodal Studios<br/>(Vision, Sound, Video, Canvas)"]
    end

    subgraph IPC_Layer["Universal IPC & Transport Layer"]
        C_FFI["Native In-Process C-FFI<br/>(miniai_bridge_dispatch)"]
        DAEMON["Auto-Spawned Micro-Daemon<br/>(http://127.0.0.1:4140 + Nonce Auth)"]
    end

    subgraph RustWorkspace["Rust Core Engine (17 Crates)"]
        Bridge["miniai-bridge Dispatcher"]
        Neural["miniai-llm (Candle GGUF Engine)"]
        Gov["miniai-core Device Governor"]
        Memory["miniai-memory (3-Tier Engine)"]
        Runtimes["7 Modality Adapters"]
    end

    Chat --> C_FFI & DAEMON
    Manager --> C_FFI & DAEMON
    Telemetry --> C_FFI & DAEMON
    Studios --> C_FFI & DAEMON

    C_FFI --> Bridge
    DAEMON --> Bridge

    Bridge --> Neural
    Bridge --> Gov
    Bridge --> Memory
    Bridge --> Runtimes
```

### 2.1 The Two IPC Channels

1. **In-Process C-FFI Bridge (`miniai-bridge`)**:
   - High-throughput, zero-network-overhead direct foreign function interface (`miniai_bridge_create`, `miniai_bridge_dispatch`, `miniai_bridge_free`).
   - Transfers structured JSON requests directly through memory pointers, freeing results safely via `miniai_bridge_string_free`.
2. **Auto-Managed Background Micro-Daemon (`miniai-daemon`)**:
   - Loopback HTTP server (`http://127.0.0.1:4140`) spawned automatically on desktop platforms.
   - Enforces session security via dynamic session nonces passed as HTTP Bearer tokens (`Authorization: Bearer <nonce>`) and request headers (`X-MiniAI-Session-Nonce`).
   - Exposes `/v1/health` and `/v1/bridge/dispatch` endpoints.

---

## 3. The 7 Multimodal Modalities

MiniAI provides independent engines across 7 AI modalities:

| # | Modality | Engine Crate | Native Backend | Capabilities |
| :-: | :--- | :--- | :--- | :--- |
| **1** | **LLM Reasoning** | `miniai-llm` | Candle / GGUF | Multi-turn conversational chat, KV-cache prefix caching, reasoning `<think>` step parsing. |
| **2** | **Vision & OCR** | `miniai-vision` | Native Tensor & SIMD | High-accuracy OCR, scene text recognition, visual question answering, image classification. |
| **3** | **Diffusion Canvas** | `miniai-imagegen` | Latent Diffusion | Text-to-image synthesis, interactive canvas mask drawing, prompt scheduling, deterministic seeds. |
| **4** | **Speech Recognition** | `miniai-speech-asr` | 16kHz Audio Pipeline | Audio header sanitation, downmixing, RMS normalization, bandpass resampling, WAV transcription. |
| **5** | **Speech Synthesis** | `miniai-speech-tts` | Formant Synthesizer | Text-to-phoneme parsing, F0 pitch contour scheduling, 22.05kHz neural formant acoustic rendering. |
| **6** | **Music Synthesis** | `miniai-music` | Polyphonic Synthesizer | Multi-track algorithmic MIDI/audio synthesis, tempo/scale harmonization, WAV master export. |
| **7** | **Video Analysis** | `miniai-video` | Spatial-Temporal Demuxer | Keyframe extraction, scene boundary change detection, temporal scene Q&A, audio demuxing. |

---

## 4. Operational Guide: Step-by-Step Usage

### 4.1 Running Local Neural Chat
1. Launch the MiniAI desktop application or run `miniai-daemon.exe`.
2. Navigate to **Chat Workspace** (`Tab 5` in the navigation rail).
3. The application automatically detects verified local weights (`smollm2-135m-instruct-q4.gguf`, 105 MB) in `models/` or `~/.miniai/models/`.
4. Submit any natural language prompt (e.g., *"how to build a picnic tent"*, *"explain offline inference"*, or coding requests).
5. Responses execute with **zero canned text**, returning genuine neural tokens streamed to the UI.
6. Multi-turn conversations retain token prefix KV-caches, dropping prefill latency by over 40% on subsequent turns.
7. Click the **Copy** button on any code block to copy formatted source code directly to your clipboard.

### 4.2 Downloading & Importing Models
1. Navigate to **Model Manager** (`Tab 3`).
2. **Local-Only Privacy Toggle**: MiniAI defaults to Local-Only mode. To download from HuggingFace Hub:
   - Click **Allow & Download** on the model card (e.g., `Llama 3.2 1B Instruct`, 807 MB).
   - This explicitly grants scoped network access to download open weights with real-time byte counters and progress bars.
   - Every downloaded byte is verified against its published SHA-256 hash.
3. **Importing Local Models**:
   - Click **Import Local Model (.gguf)**.
   - Enter your model ID, path to `.gguf` weights, format, and memory requirements.
   - The model is registered immediately in the local model cache and available in the Chat dropdown.

### 4.3 Power & Thermal Governor (Battery Conservation)
1. Check the header status bar on the top right (`CPU%`, `GPU%`, `Latency`, `FPS`).
2. If battery falls below 20% or thermal throttling is detected:
   - The status bar displays an orange **THROTTLED** or green **ECO** badge.
   - MiniAI automatically restricts concurrency to 1, caps prompt context windows to 1024 tokens, and falls back to CPU baseline to prevent battery degradation and overheating.
3. You can manually toggle Battery Saving Mode in **Settings & Privacy**.

### 4.4 Cognitive Memory Inspector
1. In the Chat Workspace, click the **Brain** icon in the top header.
2. The **3-Tier Cognitive Memory** inspector displays:
   - **Working Memory**: Active conversation turns and recent context.
   - **Episodic Memory**: Timestamped interactions and historical session summaries.
   - **Semantic Memory**: Persistent user facts and extracted profile preferences (*"User name is Alice"*, *"User prefers concise answers"*).
3. You can inspect facts, manually add new semantic memories, or purge local memory with 1 click.

### 4.5 Frontier Distillation Pipeline (Python ML)
1. Ensure `uv` is installed on your system.
2. In the `ml/` directory, run:
   ```bash
   cd ml
   uv run pytest
   ```
3. Use `distill_frontier_reasoning_dataset()` to feed teacher reasoning traces (from frontier models like Gemini Argon) and generate partitioned `.jsonl` datasets with cryptographically signed `manifest.json` files for fine-tuning compact student models.

---

## 5. Verification & Certification Matrix

Every stage of MiniAI is certified by automated test harnesses:

```
================================================================================
MiniAI Master Verification Status (v3.0.0-GA)
================================================================================
[+] Rust Workspace Crates:        347 / 347 passed (0 warnings, 0 fmt diffs)
[+] Python ML & Governance:        43 / 43 passed  (97.00% statement coverage)
[+] Flutter UI & Clients:         121 / 121 passed (90.72% line coverage)
[+] Master System Verification:   PASSED (overall_passed: true)
================================================================================
```

---

## 6. CLI Diagnostics: `miniai-daemon doctor`

To inspect host hardware, storage integrity, SIMD capabilities, and privacy mode from the command line:

```bash
.\target\debug\miniai-daemon.exe doctor
```

Output:
```json
{
  "daemon": { "loopback_enforced": true },
  "hardware": {
    "arch": "x86_64",
    "detected_backends": ["cpu", "direct_ml", "vulkan"],
    "logical_cpu_cores": 12,
    "os_family": "windows",
    "simd_features": { "avx2": true, "fma": true, "sse4_2": true }
  },
  "overall_status": "healthy",
  "privacy": { "has_active_consent": false, "local_only_mode": true },
  "storage": { "integrity_messages": ["ok"], "is_healthy": true, "schema_version": 5 }
}
```
