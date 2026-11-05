# MiniAI — Skills-to-Stage Index

This index maps all 18 reusable Antigravity skills (`.agents/skills/*/SKILL.md`) to the 11 implementation stages (`Stage 0` through `Stage 10`).

> **Note:** Skills are on-demand procedural runbooks (`SKILL.md` + `references/` + `examples/`). They are distinct from specialist subagents (`independent-qa-auditor`, `arch-security-reviewer`).

| # | Skill ID | Domain | S0 | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 |
| :- | :--- | :--- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| 1 | `miniai-architecture` | System Architecture & ADRs | X | X | X | X | X | X | X | X | X | X | X |
| 2 | `miniai-rust-core` | Rust Core Services & Crates | X | X | X | X | X | X | X | X | X | X | X |
| 3 | `miniai-flutter-ui` | Flutter Cross-Platform UI | X | X | X | X | X | X | X | X | X | X | X |
| 4 | `miniai-python-ml` | Python & PyTorch ML Tooling | X | . | . | . | X | X | X | X | X | . | X |
| 5 | `miniai-model-runtimes` | Pluggable Modality Runtimes | . | . | X | X | . | . | X | X | X | . | X |
| 6 | `miniai-training-distillation` | Fine-Tuning & Distillation | . | . | . | . | X | . | . | . | . | . | . |
| 7 | `miniai-multimodal` | Vision, Image, Audio, Music, Video | . | . | . | . | . | . | X | X | X | . | X |
| 8 | `miniai-evaluation-benchmarks` | Evaluation & Benchmarking | . | . | . | X | X | . | X | X | X | . | X |
| 9 | `miniai-hardware-optimization` | Hardware Probing & Acceleration | . | . | X | X | . | . | X | X | X | X | X |
| 10 | `miniai-offline-first` | Offline-First & Local Privacy | X | X | X | X | X | X | X | X | X | X | X |
| 11 | `miniai-security-privacy` | Security, Integrity & Consent | X | X | X | X | X | X | X | X | X | X | X |
| 12 | `miniai-containers-dev` | Containers & Reproducible Dev | X | . | . | . | . | . | . | . | . | X | X |
| 13 | `miniai-github-actions` | Multi-OS CI/CD Pipelines | X | . | . | . | . | . | . | . | . | . | X |
| 14 | `miniai-testing-qa` | Multi-Tier Testing & Coverage | X | X | X | X | X | X | X | X | X | X | X |
| 15 | `miniai-release-certification` | Packaging & Release Certification | X | . | . | . | . | . | . | . | . | . | X |
| 16 | `miniai-documentation-logs` | Append-Only Logs & Living Docs | X | X | X | X | X | X | X | X | X | X | X |
| 17 | `miniai-staged-implementation` | Staged Execution Loop & Gates | X | X | X | X | X | X | X | X | X | X | X |
| 18 | `miniai-independent-audit` | Independent QA & Security Audit | X | X | X | X | X | X | X | X | X | X | X |

