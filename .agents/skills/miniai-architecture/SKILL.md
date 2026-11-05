---
name: miniai-architecture
description: >-
  Guides architectural design, cross-language boundary contracts (Rust core, Flutter UI, Python ML),
  and Architectural Decision Record (ADR) authoring for MiniAI. Activate when designing new subsystems,
  defining API/IPC contracts, adding modality adapters, or updating ARCHITECTURE.md and ADR.md.
---

# MiniAI Architecture Skill

Use this skill whenever designing or modifying subsystem boundaries, cross-language data contracts, or ADRs.

## Core Architectural Invariants
1. **Tri-Layer Separation**:
   - **Rust Core (`crates/`)**: Process supervisor, SQLite metadata store, model registry, hardware probing, privacy/consent enforcement, and runtime adapter orchestration.
   - **Flutter UI (`ui/miniai_app/`)**: Cross-platform presentation, user consent dialogs, accessibility, and reactive state consumption.
   - **Python ML (`ml/`)**: Training, LoRA/QLoRA fine-tuning, knowledge distillation, dataset curation, and evaluation harnesses.
2. **Modality Runtime Independence**:
   - Every modality (`Llm`, `Vision`, `ImageGeneration`, `SpeechAsr`, `SpeechTts`, `Music`, `Video`) implements `ModalityAdapter` independently.
   - Never couple unrelated model architectures into a single monolithic runtime process.
3. **CPU-First & Consent-Gated Cloud**:
   - All features must degrade gracefully to local CPU execution.
   - Cloud/distributed compute requires an explicit `CloudConsentToken` and `LocalOnlyPrivacyMode == false`.

## Workflow Steps
1. Inspect [`ARCHITECTURE.md`](../../../ARCHITECTURE.md) and [`ADR.md`](../../../ADR.md).
2. Read the contract reference in [`references/boundary_contracts.md`](./references/boundary_contracts.md).
3. Define data structures in Rust first (`crates/miniai-core`), ensuring deterministic JSON serialization (`serde`).
4. Mirror contract types in Dart (`ui/miniai_app/lib/src/models/`) and Python (`ml/src/miniai_ml/contracts.py`) with contract unit tests in all three languages.
5. Record any new architectural trade-off in `ADR.md` using the standard ADR template.

## Verification
- Run `cargo test --workspace` to validate Rust contract serialization.
- Run `uv run pytest` in `ml/` and `flutter test` in `ui/miniai_app/` to verify cross-language schema parity.

