"""Unit and property tests for cryptographic provenance, adapter packaging, and model registry export."""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import pytest
import torch
import torch.nn as nn

from miniai_ml.contracts import AcceleratorBackend, Modality
from miniai_ml.provenance import (
    TrainingJobProvenance,
    create_job_provenance,
    create_registry_manifest,
    export_adapter_package,
    export_merged_model,
    save_registry_manifest,
    verify_checkpoint_provenance,
)
from miniai_ml.training import LoraConfig, TrainingConfig


def _create_dummy_file(path: Path, content: bytes = b"dummy model weight data") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(content)
    return path


def test_provenance_validation() -> None:
    """Verifies constraint enforcement and validation on TrainingJobProvenance."""
    valid_sha = "a" * 64
    valid_prov = TrainingJobProvenance(
        job_id="ft-20261106-001",
        method="lora",
        base_model_id="tiny-llm-v1",
        base_model_sha256=valid_sha,
        dataset_sha256=valid_sha,
        seed=42,
        device="cpu",
        metrics={"train_loss": 0.5, "val_loss": 0.6},
    )
    assert valid_prov.job_id == "ft-20261106-001"
    assert valid_prov.method == "lora"

    # Validation errors
    with pytest.raises(ValueError, match="job_id cannot be empty"):
        TrainingJobProvenance(
            job_id="",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="Unsafe path segment"):
        TrainingJobProvenance(
            job_id="../bad_job_id",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="Invalid training method"):
        TrainingJobProvenance(
            job_id="job1",
            method="unsupported_method",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="base_model_id cannot be empty"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="  ",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="must be exactly 64"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="m",
            base_model_sha256="short_hash",
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="seed must be >= 0"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=-1,
            device="cpu",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="device cannot be empty"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="",
            metrics={"l": 1.0},
        )

    with pytest.raises(ValueError, match="metrics dictionary cannot be empty"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={},
        )

    with pytest.raises(ValueError, match="must be exactly 64"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
            adapter_sha256="bad_adapter_sha",
        )

    with pytest.raises(ValueError, match="adapter_size_bytes must be greater than 0"):
        TrainingJobProvenance(
            job_id="job1",
            method="lora",
            base_model_id="m",
            base_model_sha256=valid_sha,
            dataset_sha256=valid_sha,
            seed=0,
            device="cpu",
            metrics={"l": 1.0},
            adapter_size_bytes=0,
        )


def test_provenance_json_roundtrip_and_file_io() -> None:
    """Verifies serialization and deserialization of provenance manifests."""
    valid_sha = "b" * 64
    prov = TrainingJobProvenance(
        job_id="ft-job-123",
        method="distillation",
        base_model_id="student-base",
        base_model_sha256=valid_sha,
        dataset_sha256=valid_sha,
        seed=100,
        device="cpu",
        metrics={"train_loss": 0.42, "kd_loss": 0.12},
        hyperparameters={"epochs": 5, "lr": 0.001},
    )

    data = prov.to_dict()
    assert data["job_id"] == "ft-job-123"
    assert data["method"] == "distillation"

    from_dict = TrainingJobProvenance.from_dict(data)
    assert from_dict.job_id == prov.job_id
    assert from_dict.metrics == prov.metrics

    with tempfile.TemporaryDirectory() as tmp_dir:
        file_path = Path(tmp_dir) / "sub" / "provenance.json"
        saved_path = prov.save_to_file(file_path)
        assert saved_path.exists()

        loaded = TrainingJobProvenance.from_file(saved_path)
        assert loaded.job_id == prov.job_id
        assert loaded.base_model_sha256 == valid_sha

        with pytest.raises(FileNotFoundError):
            TrainingJobProvenance.from_file(Path(tmp_dir) / "does_not_exist.json")


def test_create_job_provenance_and_export_adapter_package() -> None:
    """Verifies automatic hashing in create_job_provenance and adapter packaging."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_path = Path(tmp_dir)
        base_file = _create_dummy_file(tmp_path / "base_model.bin", b"base model weights")
        dataset_file = _create_dummy_file(tmp_path / "dataset.jsonl", b'{"x":1}\n{"x":2}\n')

        train_cfg = TrainingConfig(learning_rate=1e-3, batch_size=4)
        lora_cfg = LoraConfig(r=4, lora_alpha=8.0, target_modules=["fc1"])

        prov = create_job_provenance(
            job_id="ft-test-01",
            method="lora",
            base_model_id="test-base-v1",
            base_model_path=base_file,
            dataset_path=dataset_file,
            seed=42,
            device="cpu",
            metrics={"train_loss": 0.8},
            training_config=train_cfg,
            lora_config=lora_cfg,
        )

        assert len(prov.base_model_sha256) == 64
        assert len(prov.dataset_sha256) == 64
        assert prov.hyperparameters["training"]["learning_rate"] == 1e-3
        assert prov.hyperparameters["lora"]["r"] == 4

        # Export adapter package
        adapter_state = {"lora_A.weight": torch.randn(4, 8), "lora_B.weight": torch.randn(8, 4)}
        adapter_out_dir = tmp_path / "exported_adapter"
        adapter_path, prov_path = export_adapter_package(
            output_dir=adapter_out_dir,
            adapter_state_dict=adapter_state,
            provenance=prov,
        )

        assert adapter_path.exists()
        assert prov_path.exists()
        assert prov.adapter_sha256 is not None
        assert len(prov.adapter_sha256) == 64
        assert prov.adapter_size_bytes is not None
        assert prov.adapter_size_bytes > 0

        # Verify integrity
        assert prov.verify_integrity(
            base_model_path=base_file,
            dataset_path=dataset_file,
            adapter_path=adapter_path,
        )

        # Tampering base model triggers integrity error
        tampered_base = _create_dummy_file(tmp_path / "tampered.bin", b"corrupted bytes")
        with pytest.raises(ValueError, match="Base model SHA-256 mismatch"):
            prov.verify_integrity(base_model_path=tampered_base)

        # Tampering dataset triggers integrity error
        with pytest.raises(ValueError, match="Dataset SHA-256 mismatch"):
            prov.verify_integrity(dataset_path=tampered_base)

        # Tampering adapter triggers integrity error
        with pytest.raises(ValueError, match="Adapter SHA-256 mismatch"):
            prov.verify_integrity(adapter_path=tampered_base)


def test_export_merged_model() -> None:
    """Verifies merged model state dict export and checksum computation."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        model = nn.Linear(4, 2)
        out_path = Path(tmp_dir) / "merged_weights.pt"
        saved_file, digest, size = export_merged_model(model, out_path)

        assert saved_file.exists()
        assert len(digest) == 64
        assert size > 0


def test_create_and_save_registry_manifest() -> None:
    """Verifies Stage 2 Model Registry compatibility."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        model_file = _create_dummy_file(Path(tmp_dir) / "my_model.pt", b"model weights tensor")

        manifest = create_registry_manifest(
            model_id="finetuned-llama-3b",
            name="Fine-Tuned LLaMA 3B",
            model_file_path=model_file,
            modality=Modality.LLM,
            version="1.1.0",
            description="Fine-tuned for domain tasks",
            parameter_count="3B",
            recommended_backend=AcceleratorBackend.CPU,
        )

        assert manifest["model_id"] == "finetuned-llama-3b"
        assert manifest["name"] == "Fine-Tuned LLaMA 3B"
        assert manifest["modality"] == "llm"
        assert manifest["version"] == "1.1.0"
        assert len(manifest["expected_sha256"]) == 64
        assert manifest["expected_size_bytes"] == len(b"model weights tensor")
        assert manifest["recommended_backend"] == "cpu"

        out_json = Path(tmp_dir) / "manifest.json"
        saved = save_registry_manifest(manifest, out_json)
        assert saved.exists()

        content = json.loads(saved.read_text(encoding="utf-8"))
        assert content["model_id"] == "finetuned-llama-3b"

        # Missing model file raises FileNotFoundError
        with pytest.raises(FileNotFoundError):
            create_registry_manifest("bad-id", "name", Path(tmp_dir) / "missing.pt")


def test_verify_checkpoint_provenance_directory() -> None:
    """Verifies loading and verifying checkpoint_provenance.json from directory."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        chk_dir = Path(tmp_dir) / "checkpoints"
        base_file = _create_dummy_file(Path(tmp_dir) / "base.bin", b"base")
        ds_file = _create_dummy_file(Path(tmp_dir) / "ds.jsonl", b"ds")

        prov = create_job_provenance(
            job_id="chk-job-01",
            method="full",
            base_model_id="base-id",
            base_model_path=base_file,
            dataset_path=ds_file,
            seed=7,
            device="cpu",
            metrics={"train_loss": 0.1},
        )
        export_adapter_package(chk_dir, {"w": torch.ones(2)}, prov)

        loaded_prov = verify_checkpoint_provenance(
            checkpoint_dir=chk_dir,
            base_model_path=base_file,
            dataset_path=ds_file,
        )
        assert loaded_prov.job_id == "chk-job-01"

        # Missing checkpoint_provenance.json
        empty_dir = Path(tmp_dir) / "empty"
        empty_dir.mkdir()
        with pytest.raises(FileNotFoundError, match="checkpoint_provenance.json not found"):
            verify_checkpoint_provenance(empty_dir)
