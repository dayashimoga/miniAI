# MiniAI — Agent Governance & Customization Manifest (`AGENTS.md`)

This file is automatically loaded by Antigravity as the root hierarchical rule set for the `g:\miniAI` workspace. All primary agents and specialist subagents must obey these rules at all times.

---

## 1. Roles & Separation of Concerns

When operating in this repository, the engineering workflow enforces three distinct leadership roles:

1. **Principal Architect**:
   - Owns system modularity, cross-language contracts (`Rust` <-> `Flutter` <-> `Python`), `ARCHITECTURE.md`, and `ADR.md`.
   - Enforces decoupled model runtime adapters, local-first privacy guarantees, and CPU-first execution.
2. **Engineering Lead**:
   - Owns staged implementation (`Stage -> Phase -> Subphase -> Tasks -> Acceptance Tests`), clean code delivery, containerization, and CI/CD pipelines.
   - Enforces `CLAUDE.md` and `GLOBAL_ENGINEERING_CONTRACT.md`.
3. **Independent QA Lead (`independent-qa-auditor` & `arch-security-reviewer` subagents)**:
   - Independently verifies every stage gate using real command execution, coverage measurement, security checks, and append-only log validation.
   - Has veto power over stage advancement if any test fails, coverage falls below 90% on critical logic, or mock/placeholder code is detected.

> **Important Distinction (Agents vs. Skills):**
> - **Subagents** (invoked via `invoke_subagent`) are independent execution contexts with dedicated roles (e.g., `independent-qa-auditor`, `arch-security-reviewer`).
> - **Skills** (`.agents/skills/*/SKILL.md`) are modular procedural runbooks, reference guides, and verification scripts loaded on demand by an agent. Skills are **not** separate agents.

---

## 2. Mandatory Workspace Rules (`.agents/rules/`)

Modular rules reside in `.agents/rules/` and govern specific domains:
- [`.agents/rules/01-engineering-contract.md`](./.agents/rules/01-engineering-contract.md) — Core behavior, zero-mock policy, append-only logs.
- [`.agents/rules/02-rust-standards.md`](./.agents/rules/02-rust-standards.md) — Rust 2021/2024 idioms, error handling (`thiserror`), safety, and testing.
- [`.agents/rules/03-flutter-standards.md`](./.agents/rules/03-flutter-standards.md) — Flutter cross-platform UI, state management, accessibility, and widget testing.
- [`.agents/rules/04-python-ml-standards.md`](./.agents/rules/04-python-ml-standards.md) — `uv`-managed Python, PyTorch, deterministic seeds, type checking, and `pytest`.
- [`.agents/rules/05-security-and-privacy.md`](./.agents/rules/05-security-and-privacy.md) — Local-only privacy mode, zero secret leakage, SHA-256 artifact integrity, and consent gates.
- [`.agents/rules/06-testing-and-stage-gates.md`](./.agents/rules/06-testing-and-stage-gates.md) — >=90% coverage on critical logic, 100% passing tests, and stage gate evidence reporting.

---

## 3. Workspace Skills Catalog & Skills-to-Stage Index

All 18 project skills are located under `.agents/skills/<skill-name>/SKILL.md`. Activate the relevant skill(s) before starting any stage or subphase:

| Skill Name | Path | Primary Domain | Applicable Stages |
| :--- | :--- | :--- | :--- |
| `miniai-architecture` | `.agents/skills/miniai-architecture/SKILL.md` | System architecture, ADRs, cross-language contracts | Stages 0–10 |
| `miniai-rust-core` | `.agents/skills/miniai-rust-core/SKILL.md` | Rust workspace, async services, SQLite storage, event bus | Stages 0–10 |
| `miniai-flutter-ui` | `.agents/skills/miniai-flutter-ui/SKILL.md` | Flutter desktop/mobile UI, accessibility, state binding | Stages 0–10 |
| `miniai-python-ml` | `.agents/skills/miniai-python-ml/SKILL.md` | Python/PyTorch tooling, `uv` workflows, dataset/ML scripts | Stages 0, 4, 5, 6, 7, 8 |
| `miniai-model-runtimes` | `.agents/skills/miniai-model-runtimes/SKILL.md` | Pluggable model runtime adapters (LLM, Vision, Audio, Video) | Stages 2, 3, 6, 7, 8 |
| `miniai-training-distillation` | `.agents/skills/miniai-training-distillation/SKILL.md` | LoRA/QLoRA fine-tuning, knowledge distillation, checkpoints | Stage 4 |
| `miniai-multimodal` | `.agents/skills/miniai-multimodal/SKILL.md` | Vision, image gen/edit, speech ASR/TTS, music, video pipelines | Stages 6, 7, 8 |
| `miniai-evaluation-benchmarks` | `.agents/skills/miniai-evaluation-benchmarks/SKILL.md` | Model benchmarking, latency/throughput, quality metrics | Stages 3, 4, 6, 7, 8, 10 |
| `miniai-hardware-optimization` | `.agents/skills/miniai-hardware-optimization/SKILL.md` | CPU-first execution, GPU/NPU probing, memory budgets | Stages 2, 3, 6, 7, 8, 9 |
| `miniai-offline-first` | `.agents/skills/miniai-offline-first/SKILL.md` | Local-only privacy mode, offline resilience, cache integrity | Stages 0–10 |
| `miniai-security-privacy` | `.agents/skills/miniai-security-privacy/SKILL.md` | Threat modeling, secret scanning, consent tokens, sandboxing | Stages 0–10 |
| `miniai-containers-dev` | `.agents/skills/miniai-containers-dev/SKILL.md` | Docker, Compose, Devcontainers, reproducible environments | Stages 0, 9, 10 |
| `miniai-github-actions` | `.agents/skills/miniai-github-actions/SKILL.md` | Multi-OS CI/CD (Windows, Linux, macOS, Android, iOS) | Stages 0, 10 |
| `miniai-testing-qa` | `.agents/skills/miniai-testing-qa/SKILL.md` | Multi-tier testing, coverage enforcement (>=90%), regression | Stages 0–10 |
| `miniai-release-certification` | `.agents/skills/miniai-release-certification/SKILL.md` | Cross-platform packaging, SBOM, checksums, release gates | Stages 0, 10 |
| `miniai-documentation-logs` | `.agents/skills/miniai-documentation-logs/SKILL.md` | Append-only `TODO.md`/`CHANGELOG.md` and living docs | Stages 0–10 |
| `miniai-staged-implementation` | `.agents/skills/miniai-staged-implementation/SKILL.md` | Stage -> Phase -> Subphase execution loop & gate reports | Stages 0–10 |
| `miniai-independent-audit` | `.agents/skills/miniai-independent-audit/SKILL.md` | Independent QA & security auditing before stage sign-off | Stages 0–10 |

---

## 4. Critical Operational Prohibitions

1. **NEVER** overwrite, delete, reorder, or edit existing lines in `TODO.md` or `CHANGELOG.md`. Always append new dated entries at the bottom.
2. **NEVER** commit secrets, API tokens, private keys, or credentials to git or workflow logs.
3. **NEVER** claim a remote Git/GitHub operation or test execution succeeded without running a verification command and inspecting the actual output.
4. **NEVER** advance from Stage $N$ to Stage $N+1$ until Stage $N$ passes all acceptance tests and independent QA audit checks.

