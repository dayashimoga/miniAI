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

## 3. Troubleshooting & Recovery

- **"PrivacyModeBlocked: Outbound cloud execution is prohibited..."**:
  - This confirms `Local-Only Privacy Mode` is protecting your workspace. To run a task on a remote cloud worker (Stage 9), disable Local-Only Privacy Mode and grant an explicit consent token, or run the task locally on CPU/GPU.
- **"InsufficientMemory: Model requires X bytes, exceeding safe limit..."**:
  - MiniAI prevents system freezes by enforcing an 85% available RAM/VRAM admission ceiling. Choose a higher quantization level (e.g., `Q4_K_M`) or close background applications.

