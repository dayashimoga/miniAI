---
name: miniai-python-ml
description: >-
  Guides Python and PyTorch ML tooling under ml/ using uv for reproducible dependency management.
  Activate when working on dataset pipelines, fine-tuning, distillation, evaluation scripts,
  or Python governance verifiers.
---

# MiniAI Python & ML Engineering Skill

Use this skill when developing or testing Python modules in `ml/` or verification scripts in `scripts/`.

## Environment & Tooling Rules
1. **Always use `uv`**: Execute Python commands via `uv run` so dependencies and Python versions remain isolated and reproducible.
2. **CPU-First Portability**: All ML utilities must run deterministically on CPU when no discrete GPU is available.
3. **Coverage**: Maintain >= 90% measured test coverage across `ml/src/miniai_ml`.

## Workflow Steps
1. Consult [`references/uv_workflow.md`](./references/uv_workflow.md).
2. Implement typed Python modules under `ml/src/miniai_ml/`.
3. Write unit and contract tests under `ml/tests/`.
4. Execute tests with coverage from `ml/`:
   `uv run pytest --cov=miniai_ml --cov-report=term-missing`

## Verification
- `uv run pytest` must exit with code `0` and >= 90% coverage on `miniai_ml`.

