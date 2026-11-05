# Training & Distillation Provenance Manifest Specification

Every fine-tuning or distillation output directory must contain `checkpoint_provenance.json`:

```json
{
  "job_id": "ft-20261105-001",
  "method": "lora",
  "base_model_id": "miniai-tiny-llm-v1",
  "base_model_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "dataset_sha256": "a1b2c3d4e5f60123456789abcdef0123456789abcdef0123456789abcdef0123",
  "seed": 42,
  "device": "cpu",
  "metrics": {
    "train_loss": 1.12,
    "val_loss": 1.18
  }
}
```

