# Stage 2 Technical Execution Prompt: Model Registry, Downloads, Integrity, Hardware Detection & Runtime Adapters

## 1. Objective & Dependencies
- **Objective:** Deliver the local model registry (`crates/miniai-registry`), resumable HTTP/file downloader with mandatory SHA-256 verification, hardware capability detector with 85% RAM/VRAM admission control, and isolated runtime adapter supervisor (`crates/miniai-runtime`).
- **Dependencies:** Stage 1 Gate Verified.
- **Active Skills:** `miniai-rust-core`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-flutter-ui`, `miniai-offline-first`, `miniai-security-privacy`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 2.1: Model Catalog & Resumable Verified Downloads**
  - *Subphase 2.1.1:* SQLite model manifest store, staging quarantine directory (`.staging/`), HTTP `Range` resume support, and streaming SHA-256 calculation.
  - *Subphase 2.1.2:* Offline local model import wizard with digest verification before promotion to verified cache.
- **Phase 2.2: Hardware Probing & Admission Guard**
  - *Subphase 2.2.1:* CPU SIMD feature detection (`AVX2`, `AVX512`, `NEON`), system RAM & GPU VRAM probing, and 85% memory ceiling enforcement.
- **Phase 2.3: Modality Runtime Supervisor & Flutter Model Manager**
  - *Subphase 2.3.1:* Independent lifecycle supervisor for all 7 modalities with automatic CPU fallback on accelerator failure.
  - *Subphase 2.3.2:* Flutter Model Registry UI with download progress, pause/resume/cancel, and SHA-256 verification badges.

## 3. Security & Edge Cases
- Immediately quarantine and delete any downloaded or imported weight file whose SHA-256 digest does not match the manifest.
- Prevent path traversal in model IDs and file names.
- Block model downloads when `LocalOnlyPrivacyMode` is set to disallow external registry fetches, while permitting local file imports.

## 4. Acceptance Tests & Exit Criteria
- Corrupted/tampered model weights are rejected 100% of the time in unit and integration tests.
- Interrupted downloads resume from exact byte offset.
- >= 90% coverage across `miniai-registry` and `miniai-runtime`.

