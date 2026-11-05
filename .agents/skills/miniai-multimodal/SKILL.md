---
name: miniai-multimodal
description: >-
  Guides multimodal pipeline engineering across Vision, Image Generation/Editing, Speech ASR/TTS,
  Music, and Video in MiniAI (Stages 6, 7, and 8). Activate when implementing media preprocessing,
  tensor normalization, codec validation, or multimodal pipeline orchestration.
---

# MiniAI Multimodal Engineering Skill

Use this skill when building or validating vision, image generation, speech/audio, music, or video pipelines.

## Multimodal Design Rules
1. **Strict Media Input Validation**: Verify MIME types, magic bytes, image dimensions, audio sample rates, and video frame bounds before decoding media into memory.
2. **Independent Modality Execution**: Each modality pipeline executes via its registered `ModalityAdapter` without blocking the UI thread or other modalities.
3. **Deterministic Media Post-Processing**: Ensure generated images, audio waveforms, and video segments are saved atomically with SHA-256 metadata records.

## Workflow Steps
1. Consult [`references/media_pipeline_limits.md`](./references/media_pipeline_limits.md).
2. Validate input media headers and resource bounds in Rust before invoking the modality adapter.
3. Execute inference on the selected backend (defaulting to CPU if no accelerator is active).
4. Persist output artifacts and emit structured completion events to the UI shell.

## Verification
- Run multimodal contract and boundary tests in `cargo test` and `uv run pytest`.

