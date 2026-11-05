# MiniAI

**Production-Grade, Modular, Local-First Multimodal AI Platform with Hybrid Local/Cloud Compute and Independently Pluggable Model Architectures.**

---

## Overview

**MiniAI** combines a memory-safe **Rust** core (`crates/`), a responsive cross-platform **Flutter** UI (`ui/miniai_app/`) targeting **Windows, Linux, macOS, Android, and iOS**, and **Python/PyTorch** ML tooling (`ml/`) for dataset curation, LoRA/QLoRA fine-tuning, knowledge distillation, and evaluation.

### Architectural Pillars
- **Independently Pluggable Modalities**: LLM Text (`llm`), Vision Understanding (`vision`), Image Generation/Editing (`image_generation`), Speech Recognition (`speech_asr`), Neural TTS (`speech_tts`), Music (`music`), and Video (`video`) operate through isolated `ModalityAdapter` runtimes—never forced to share a monolithic runtime.
- **CPU-First & Hardware Adaptive**: Every workflow executes deterministically on standard CPUs (`AcceleratorBackend::Cpu`) and dynamically utilizes optional accelerators (`CUDA`, `Vulkan`, `Metal`, `DirectML`, `ROCm`, `CoreML`, `NNAPI`) with automatic CPU fallback.
- **Local-Only Privacy Mode by Default**: Outbound cloud inference and telemetry are blocked at the core policy layer unless the user explicitly disables `LocalOnlyPrivacyMode` and grants a scoped, time-limited `CloudConsentToken`.
- **Zero Mock Policy**: Every UI screen, toggle, and service endpoint is backed by real, tested domain logic.

---

## Documentation Index

| Document | Purpose |
| :--- | :--- |
| [`GLOBAL_ENGINEERING_CONTRACT.md`](./GLOBAL_ENGINEERING_CONTRACT.md) | Binding engineering contract, coverage targets (`>=90%`), and stage gate rules |
| [`CLAUDE.md`](./CLAUDE.md) | Core behavioral guidelines integrated with MiniAI rules |
| [`AGENTS.md`](./AGENTS.md) | Antigravity agent roles, rules, and 18-skill matrix |
| [`REQUIREMENTS.md`](./REQUIREMENTS.md) | Functional (`FR`) and non-functional (`NFR`) system requirements |
| [`ARCHITECTURE.md`](./ARCHITECTURE.md) | Tri-layer architecture, modality isolation, and privacy state machine |
| [`ADR.md`](./ADR.md) | Architectural Decision Records (`ADR-001` through `ADR-005`) |
| [`ROADMAP.md`](./ROADMAP.md) | Master 11-stage roadmap (`Stage 0` through `Stage 10`) |
| [`IMPLEMENTATION_PLAN.md`](./IMPLEMENTATION_PLAN.md) | Detailed `Stage -> Phase -> Subphase -> Tasks -> Acceptance Tests` plan |
| [`CODEBASE_GUIDE.md`](./CODEBASE_GUIDE.md) | Repository layout and subsystem guide |
| [`DEVELOPMENT_SETUP.md`](./DEVELOPMENT_SETUP.md) | Host & container setup and verification commands |
| [`USER_GUIDE.md`](./USER_GUIDE.md) | End-user operational and privacy guide |
| [`TESTING.md`](./TESTING.md) | Multi-tier testing strategy and coverage measurement |
| [`SECURITY.md`](./SECURITY.md) | Threat model, SHA-256 verification, and consent security policy |
| [`CONTRIBUTING.md`](./CONTRIBUTING.md) | Contributor workflow and append-only log rules |
| [`RELEASE_CERTIFICATION.md`](./RELEASE_CERTIFICATION.md) | Cross-platform release and stage gate certification checklist |
| [`TODO.md`](./TODO.md) | Strictly append-only chronological engineering task log |
| [`CHANGELOG.md`](./CHANGELOG.md) | Strictly append-only chronological release and milestone log |

---

## Quick Verification

```powershell
# Run Rust Core tests
cargo test --workspace

# Run Python ML & Governance tests (>=90% coverage enforced)
Push-Location ml; uv run pytest --cov=miniai_ml --cov-fail-under=90; Pop-Location

# Run Flutter UI analysis & widget tests
Push-Location ui/miniai_app; flutter analyze --no-pub; flutter test --coverage; Pop-Location

# Run Master Stage 0 Gate Verifier
uv run --project ml python scripts/verify_stage0.py
```

