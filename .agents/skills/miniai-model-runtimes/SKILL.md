---
name: miniai-model-runtimes
description: >-
  Guides the design and integration of independently pluggable model runtime adapters
  (LLM, Vision, Image, Audio, Music, Video) in MiniAI. Activate during Stages 2, 3, 6, 7, and 8
  when adding or modifying model runtime adapters.
---

# MiniAI Pluggable Model Runtimes Skill

Use this skill when implementing or auditing runtime adapters for any AI modality.

## Runtime Independence Rules
1. **Independent Lifecycle**: Each modality adapter (`Llm`, `Vision`, `ImageGeneration`, `SpeechAsr`, `SpeechTts`, `Music`, `Video`) is loaded, health-checked, and unloaded independently.
2. **No Forced Monolith**: Never require a diffusion runtime or audio synthesizer to run inside an LLM runtime engine.
3. **SHA-256 Verification Before Load**: Verify model artifact SHA-256 digests before passing file handles to any native runtime.

## Workflow Steps
1. Review [`references/runtime_adapter_matrix.md`](./references/runtime_adapter_matrix.md).
2. Implement the `ModalityAdapter` trait in a dedicated module or crate.
3. Register the adapter with `RuntimeRegistry` and verify capability negotiation (CPU fallback + optional hardware acceleration).
4. Write contract tests verifying isolation, health check reporting, and graceful error handling on corrupted or missing weights.

## Verification
- Run `cargo test --workspace` to verify adapter registration, capability negotiation, and isolation.

