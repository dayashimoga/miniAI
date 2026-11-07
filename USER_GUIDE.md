# MiniAI — User Guide (`USER_GUIDE.md`)

**Document ID:** `MINIAI-USER-001`  
**Version:** `1.0.0`

---

## 1. Welcome to MiniAI

**MiniAI** is a local-first, privacy-preserving multimodal AI platform that runs on your desktop or mobile device (**Windows, Linux, macOS, Android, and iOS**).

### Key Principles for Users
1. **Works Out-of-the-Box on CPU**: You do not need an expensive discrete GPU to run MiniAI. Every modality includes a verified CPU execution path, while automatically utilizing GPUs or NPUs if your device has them.
2. **Local-Only Privacy Mode by Default**: By default, MiniAI operates in **Local-Only Privacy Mode**. Your prompts, files, images, audio, and models never leave your device.
3. **Independently Pluggable Modalities**: Load only the modalities you need—Text LLM, Vision Understanding, Image Generation, Speech Recognition (ASR), Text-to-Speech (TTS), Music, or Video.
4. **Explicit Consent for Hybrid Cloud**: Cloud GPU offload is completely disabled unless you explicitly turn off Local-Only Privacy Mode and grant a scoped, time-limited consent token for a specific task.

---

## 2. Launching the Application

### Running the Desktop/Mobile UI (Development Build)
```powershell
cd ui/miniai_app
flutter run
```

### Inspecting System Status in the Dashboard
When MiniAI opens, the **Platform Control & Modality Dashboard** displays:
1. **Privacy & Consent Banner**:
   - Shows `Local-Only Privacy Mode: ACTIVE` (green shield) by default.
   - Allows toggling `Force CPU-Only Execution` at any time.
   - Provides explicit controls to test, grant, or immediately revoke a scoped `CloudConsentToken` when hybrid mode is inspected.
2. **Hardware Profile Card**:
   - Displays detected logical CPU cores, total/available RAM, 85% safe model memory admission limit, and active execution backend (`CPU` or detected accelerator).
3. **Independent Modality Runtime Registry**:
   - Lists each registered modality adapter (`LLM`, `Vision`, `Image Generation`, `Speech ASR`, `Speech TTS`, `Music`, `Video`), its process isolation status, supported backends, and live health check state.

---

## 3. Workspaces & Specialized Creative Studios

MiniAI organizes all AI capabilities across dedicated, responsive workspace tabs accessible via the desktop `NavigationRail` and mobile `NavigationBar`:

### Tab 1: Platform Dashboard & Hardware Telemetry
Monitor host CPU/GPU admission budgets, real-time memory pressure, event bus telemetry, and universal CPU baseline execution.

### Tab 2: Model Registry & Cryptographic Quarantine
Browse certified offline models, inspect SHA-256 cryptographic digests, download weights with resumable downloads, and verify that corrupted files are quarantined automatically.

### Tab 3: Offline LLM Chat & Hyperparameter Tuning
Interact with local GGUF models. Tune temperature, top-p, context length, and stop sequences. Edit and branch conversation messages with export to Markdown and JSON.

### Tab 6: Training Studio (LoRA, QLoRA & Distillation)
Curate dataset splits, run micro-training on CPU, configure LoRA rank/alpha hyperparameters, and track teacher-student distillation loss curves with verifiable checkpoint provenance manifests.

### Tab 7: Knowledge Base & Offline Document RAG
Index local documents, Markdown files, and source code. Query the offline vector index with exact cosine similarity matching, and review grounded citation badges linking directly to source line numbers.

### Tab 8: Canvas Studio (Vision Understanding & Diffusion Inpainting)
Analyze images with OCR text extraction, scene captioning, and grounded VQA. Generate high-resolution images via scheduled denoising diffusion, draw interactive inpainting masks, and inspect embedded PNG cryptographic provenance chunks.

### Tab 9: Sound Studio (Speech ASR, Formant TTS & Polyphonic Music)
Transcribe speech with offline 16 kHz ASR and energy/ZCR Voice Activity Detection. Synthesize natural speech across 6 acoustic voice profiles. Compose multi-track polyphonic music (melody, harmony pad, bassline, and rhythm) in modal scales.

### Tab 10: Video Studio (Temporal Scene QA & Multimodal DAGs)
Extract keyframes safely within a 64 MB bounded buffer pool. Run grounded temporal VQA across scenes. Synthesize camera-motion animations. Build and execute composable multimodal DAG workflows (e.g., Video -> Audio Demux -> ASR -> LLM Summary -> TTS Voiceover).

### Tab 11: Hybrid Compute & Consent Inspector
For users desiring cloud GPU offload: inspect active `CloudConsentToken` authorizations, enforce HTTPS-only endpoints, monitor quota countdown timers, simulate fail-closed network errors with automatic local CPU execution fallback, and trigger the instant **Emergency Kill-Switch** (`Revoke All`).

---

## 4. Troubleshooting, Privacy Guarantees & Recovery

- **"PrivacyModeBlocked: Outbound cloud execution is prohibited..."**:
  - This confirms `Local-Only Privacy Mode` is protecting your workspace. To run a task on a remote cloud worker (Stage 9), disable Local-Only Privacy Mode and grant an explicit consent token, or run the task locally on CPU/GPU.
- **"InsufficientMemory: Model requires X bytes, exceeding safe limit..."**:
  - MiniAI prevents system freezes by enforcing an 85% available RAM/VRAM admission ceiling. Choose a higher quantization level (e.g., `Q4_K_M`) or close background applications.
- **"Database Corrupted or Quarantine Triggered"**:
  - If unexpected hardware failure or sudden power loss corrupts SQLite storage, MiniAI automatically quarantines the damaged database to a timestamped backup directory and spins up a healthy replacement database with full schema migrations preserved.


