# Stage 4 Technical Execution Prompt: Dataset Management, Fine-Tuning, Training, Distillation & Evaluation

## 1. Objective & Dependencies
- **Objective:** Deliver local dataset curation/splitting, parameter-efficient fine-tuning (LoRA/QLoRA), teacher-student knowledge distillation, cryptographic checkpoint provenance manifests, and model evaluation (`ml/src/miniai_ml` + Rust job supervisor + Flutter Training Studio).
- **Dependencies:** Stage 3 Gate Verified.
- **Active Skills:** `miniai-python-ml`, `miniai-training-distillation`, `miniai-evaluation-benchmarks`, `miniai-rust-core`, `miniai-flutter-ui`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 4.1: Dataset Curation, Schema Validation & Splitting**
  - *Subphase 4.1.1:* JSONL/CSV instruction & preference dataset validator, deduplicator, deterministic `train`/`val`/`test` splitter, and SHA-256 dataset manifest generator.
- **Phase 4.2: LoRA/QLoRA Fine-Tuning & Knowledge Distillation**
  - *Subphase 4.2.1:* CPU-compatible and GPU-accelerated PyTorch LoRA trainer and teacher-student KL-divergence distillation engine with step checkpointing.
  - *Subphase 4.2.2:* `checkpoint_provenance.json` lineage generation and automatic registration into the local Model Registry.
- **Phase 4.3: Flutter Training & Evaluation Studio**
  - *Subphase 4.3.1:* Dataset browser, training job launcher, live loss curve viewer, pause/resume controls, and side-by-side base vs. fine-tuned evaluation comparison.

## 3. Security & Edge Cases
- Reject malformed or poisoned dataset records (invalid UTF-8, missing fields, oversized rows).
- Ensure training jobs can be paused and resumed cleanly from step checkpoints without data loss.

## 4. Acceptance Tests & Exit Criteria
- Deterministic CPU micro-training and distillation runs reduce validation loss reproducibly in `pytest`.
- Checkpoint provenance manifests pass SHA-256 verification.
- >= 90% coverage across `ml/src/miniai_ml`.

