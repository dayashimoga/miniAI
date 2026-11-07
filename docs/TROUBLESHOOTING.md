# MiniAI — Cross-Platform Troubleshooting & Operations Guide

This runbook provides installation instructions, hardware tuning guides, diagnostic commands, and self-repair procedures for MiniAI across Windows, Linux, macOS, Android, and iOS.

---

## 1. Quick Start & Platform Installation

### 1.1 Windows Desktop (x86_64)
- **Minimum Requirements**: Windows 10/11 64-bit, 4 GB RAM, x86_64 CPU with SSE4.2.
- **Hardware Acceleration**: Automatic DirectML and Vulkan detection. CPU AVX2/FMA baseline guaranteed.
- **Running the Daemon**:
  ```powershell
  # Start local daemon
  .\miniai-daemon.exe

  # Run diagnostic self-check
  .\miniai-daemon.exe doctor
  ```

### 1.2 Linux Desktop (Ubuntu / Debian / Fedora)
- **Minimum Requirements**: Linux Kernel 5.4+, 4 GB RAM, GTK3 runtime (`libgtk-3-dev`, `liblzma-dev`).
- **Hardware Acceleration**: Vulkan loader (`libvulkan1`).
- **Running the App**:
  ```bash
  tar -xzvf miniai-2.0.0-rc1-linux-x86_64.tar.gz
  cd miniai-linux-x86_64
  ./miniai_app
  ```

### 1.3 macOS Desktop (Apple Silicon & Intel)
- **Minimum Requirements**: macOS 12.0 (Monterey) or later.
- **Hardware Acceleration**: Apple Metal and Accelerate SIMD.
- **Running the App**:
  ```bash
  tar -xzvf miniai-2.0.0-rc1-macos-universal.tar.gz
  open miniai_app.app
  ```

### 1.4 Android (ARM64-v8a)
- **Minimum Requirements**: Android 8.0 (API 26)+, 3 GB RAM.
- **Installation**:
  ```bash
  adb install -r miniai-2.0.0-rc1-android.apk
  ```

---

## 2. Model Downloads, Caching & Privacy Modes

### 2.1 "Remote downloads blocked in Local-Only Privacy Mode"
- **Cause**: By default, MiniAI operates in **Local-Only Privacy Mode** to prevent data leakage.
- **Resolution**:
  1. Go to **Models** in the sidebar.
  2. Locate the model (e.g. `SmolLM2 135M Instruct (GGUF Q4)` or `Llama 3.2 1B Instruct (GGUF Q4)`).
  3. Click **Download & Verify**.
  4. In the confirmation dialog, review the target URL (`huggingface.co`) and click **Allow & Download**. Public open-weight weights will download over HTTPS without exposing user data.
  5. Alternatively, click **Import Model** to load a local `.gguf` or `.onnx` file from disk.

### 2.2 Paused or Interrupted Downloads
- MiniAI uses HTTP Range requests (`ResumableDownloader`).
- If network connection drops during download, click **Resume** on the model card. Download will continue from the last received byte.

### 2.3 SHA-256 Checksum Quarantine
- When a download finishes, MiniAI streams the bytes through an SHA-256 hasher.
- If the computed digest does not match the pinned manifest, the file is immediately quarantined (`.corrupt.<epoch>`) and removed from active cache.

---

## 3. Hardware Admission Guard & Memory Ceiling

### 3.1 85% Safe RAM Admission Guard
- To prevent out-of-memory crashes or system thrashing, MiniAI enforces an 85% available RAM ceiling before loading any model weights.
- If requested model size exceeds the 85% ceiling:
  * The model manager flags the model as `Admission Rejected (RAM Insufficient)`.
  * MiniAI offers automatic fallback to a smaller quantized model (e.g. SmolLM2 135M instead of Llama 3.2 1B).

### 3.2 Freeing Memory from Loaded Models
- To release RAM held by loaded models:
  1. Open **Hardware** from the sidebar.
  2. Locate the **Live Resource Telemetry & Active Model** card.
  3. Click **Unload Model from RAM**. Memory is instantly freed and active model status transitions to `None (Memory Unloaded)`.

---

## 4. Local Storage & Database Self-Repair

### 4.1 SQLite Storage Integrity
- Storage is managed via `StorageEngine` in `miniai_storage.db` with WAL mode and atomic transactions.
- MiniAI automatically migrates schemas (Migrations 1 through 5).

### 4.2 Corrupt Database Quarantine & Recovery
- If the SQLite database experiences power loss corruption or zero-byte truncation:
  1. The storage engine catches the corruption on initialization.
  2. The damaged file is quarantined to `miniai_storage.db.corrupt.<timestamp>`.
  3. A fresh, healthy database is initialized and migrations applied automatically.
  4. System audit log records a `database_quarantined_and_reinitialized` event.

---

## 5. Built-in Diagnostics & Troubleshooting Wizard

### 5.1 CLI Doctor Command
Run the built-in diagnostic CLI command to check hardware capabilities, storage health, and daemon status:
```bash
miniai-daemon doctor
```
**Sample Output**:
```json
{
  "daemon": {
    "loopback_enforced": true
  },
  "hardware": {
    "arch": "x86_64",
    "detected_backends": ["cpu", "direct_ml", "vulkan"],
    "gpus_count": 1,
    "logical_cpu_cores": 12,
    "os_family": "windows",
    "simd_features": {
      "avx2": true,
      "fma": true,
      "neon": false,
      "sse4_2": true
    }
  },
  "overall_status": "healthy",
  "privacy": {
    "has_active_consent": false,
    "local_only_mode": true
  },
  "storage": {
    "integrity_messages": ["ok"],
    "is_healthy": true,
    "schema_version": 5
  }
}
```

### 5.2 Exporting PII-Redacted Diagnostics
- To report an issue without disclosing sensitive personal data:
  1. Click **Events** in the navigation bar.
  2. Click the **Export Redacted Diagnostics** download icon in the toolbar.
  3. A sanitized JSON bundle is generated where username file paths (`C:\Users\<user>`) and bearer tokens are automatically replaced with `<redacted_user>`.
