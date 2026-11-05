---
name: miniai-training-distillation
description: >-
  Guides dataset validation, LoRA/QLoRA fine-tuning, knowledge distillation, and checkpoint
  provenance tracking in MiniAI (Stage 4). Activate when working on training pipelines,
  hyperparameter validation, loss tracking, or model distillation workflows.
---

# MiniAI Training, Fine-Tuning & Distillation Skill

Use this skill when building or verifying dataset pipelines, LoRA/QLoRA fine-tuning jobs, or teacher-student distillation workflows.

## Key Engineering Requirements
1. **Dataset Integrity**: Validate schema, UTF-8 encoding, split ratios (`train`/`val`/`test`), and SHA-256 dataset manifests before starting any training job.
2. **Resource Budgeting**: Enforce memory and CPU/GPU limits; allow clean checkpoint pause/resume at epoch or step boundaries.
3. **Checkpoint Provenance**: Every produced checkpoint or adapter weight file must include a provenance manifest (base model hash, dataset hash, seed, hyperparameters, validation metrics).

## Workflow Steps
1. Consult [`references/training_manifest_spec.md`](./references/training_manifest_spec.md).
2. Validate training configuration and dataset splits via `miniai_ml`.
3. Execute training/distillation step loop with deterministic seeding and metric logging.
4. Export verified checkpoint manifest and run post-training evaluation.

## Verification
- Run `uv run pytest` on training and distillation test suites to confirm deterministic convergence on synthetic/micro test fixtures.

