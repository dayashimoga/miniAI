"""Cryptographic provenance tracking, adapter packaging, and model registry export.

This module provides:
1. `TrainingJobProvenance`: Cryptographic manifest tracking base model SHA-256,
   dataset SHA-256, random seed, hyperparameters, and convergence metrics.
2. Adapter packaging and checkpoint provenance persistence (`checkpoint_provenance.json`).
3. Merged model export with streaming SHA-256 hashing.
4. Model Registry manifest generation compatible with MiniAI's Stage 2 registry schema.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
import json
from pathlib import Path
from typing import Any, Dict, Optional, Tuple, Union

import torch
import torch.nn as nn

from .contracts import (
    AcceleratorBackend,
    Modality,
    compute_sha256_hex,
    validate_safe_relative_path,
    verify_sha256_digest,
)
from .dataset import compute_file_sha256
from .training import LoraConfig, TrainingConfig


@dataclass
class TrainingJobProvenance:
    """Cryptographic manifest documenting fine-tuning or distillation provenance.

    Conforms to `references/training_manifest_spec.md`.
    """

    job_id: str
    method: str
    base_model_id: str
    base_model_sha256: str
    dataset_sha256: str
    seed: int
    device: str
    metrics: Dict[str, float]
    hyperparameters: Dict[str, Any] = field(default_factory=dict)
    created_at: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )
    adapter_sha256: Optional[str] = None
    adapter_size_bytes: Optional[int] = None

    def __post_init__(self) -> None:
        trimmed_job_id = self.job_id.strip()
        if not trimmed_job_id:
            raise ValueError("job_id cannot be empty")
        validate_safe_relative_path(trimmed_job_id)

        valid_methods = ("lora", "qlora", "distillation", "full")
        if self.method not in valid_methods:
            raise ValueError(
                f"Invalid training method '{self.method}'. Must be one of: {valid_methods}"
            )

        if not self.base_model_id.strip():
            raise ValueError("base_model_id cannot be empty")

        self.base_model_sha256 = self._validate_sha256(
            self.base_model_sha256, "base_model_sha256"
        )
        self.dataset_sha256 = self._validate_sha256(
            self.dataset_sha256, "dataset_sha256"
        )

        if self.seed < 0:
            raise ValueError(f"seed must be >= 0, got {self.seed}")

        if not self.device.strip():
            raise ValueError("device cannot be empty")

        if not self.metrics:
            raise ValueError("metrics dictionary cannot be empty")

        if self.adapter_sha256 is not None:
            self.adapter_sha256 = self._validate_sha256(
                self.adapter_sha256, "adapter_sha256"
            )

        if self.adapter_size_bytes is not None and self.adapter_size_bytes <= 0:
            raise ValueError("adapter_size_bytes must be greater than 0")

    @staticmethod
    def _validate_sha256(digest: str, field_name: str) -> str:
        clean = digest.strip().lower()
        if len(clean) != 64 or any(c not in "0123456789abcdef" for c in clean):
            raise ValueError(
                f"Invalid {field_name} '{digest}': must be exactly 64 hexadecimal characters"
            )
        return clean

    def to_dict(self) -> Dict[str, Any]:
        """Serializes provenance record to JSON-compatible dictionary."""
        return asdict(self)

    def to_json(self, indent: int = 2) -> str:
        """Serializes provenance record to formatted JSON string."""
        return json.dumps(self.to_dict(), indent=indent)

    def save_to_file(self, path: Union[str, Path]) -> Path:
        """Writes provenance record atomically to file."""
        target_path = Path(path)
        target_path.parent.mkdir(parents=True, exist_ok=True)
        tmp_path = target_path.with_suffix(".tmp")
        tmp_path.write_text(self.to_json(), encoding="utf-8")
        tmp_path.replace(target_path)
        return target_path

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> TrainingJobProvenance:
        """Instantiates provenance record from dictionary."""
        return cls(
            job_id=data["job_id"],
            method=data["method"],
            base_model_id=data["base_model_id"],
            base_model_sha256=data["base_model_sha256"],
            dataset_sha256=data["dataset_sha256"],
            seed=data["seed"],
            device=data["device"],
            metrics=data["metrics"],
            hyperparameters=data.get("hyperparameters", {}),
            created_at=data.get(
                "created_at", datetime.now(timezone.utc).isoformat()
            ),
            adapter_sha256=data.get("adapter_sha256"),
            adapter_size_bytes=data.get("adapter_size_bytes"),
        )

    @classmethod
    def from_file(cls, path: Union[str, Path]) -> TrainingJobProvenance:
        """Loads and parses provenance manifest from disk."""
        target_path = Path(path)
        if not target_path.exists():
            raise FileNotFoundError(f"Provenance file not found at: {target_path}")
        content = target_path.read_text(encoding="utf-8")
        data = json.loads(content)
        return cls.from_dict(data)

    def verify_integrity(
        self,
        base_model_path: Optional[Union[str, Path]] = None,
        dataset_path: Optional[Union[str, Path]] = None,
        adapter_path: Optional[Union[str, Path]] = None,
    ) -> bool:
        """Verifies provenance digests against physical files on disk."""
        if base_model_path is not None:
            actual_base = compute_file_sha256(base_model_path)
            if actual_base != self.base_model_sha256:
                raise ValueError(
                    f"Base model SHA-256 mismatch: expected {self.base_model_sha256}, got {actual_base}"
                )

        if dataset_path is not None:
            actual_ds = compute_file_sha256(dataset_path)
            if actual_ds != self.dataset_sha256:
                raise ValueError(
                    f"Dataset SHA-256 mismatch: expected {self.dataset_sha256}, got {actual_ds}"
                )

        if adapter_path is not None and self.adapter_sha256 is not None:
            actual_adapter = compute_file_sha256(adapter_path)
            if actual_adapter != self.adapter_sha256:
                raise ValueError(
                    f"Adapter SHA-256 mismatch: expected {self.adapter_sha256}, got {actual_adapter}"
                )

        return True


def create_job_provenance(
    job_id: str,
    method: str,
    base_model_id: str,
    base_model_path: Union[str, Path],
    dataset_path: Union[str, Path],
    seed: int,
    device: str,
    metrics: Dict[str, float],
    training_config: Optional[TrainingConfig] = None,
    lora_config: Optional[LoraConfig] = None,
) -> TrainingJobProvenance:
    """Helper creating a verified `TrainingJobProvenance` from file paths and configs."""
    base_sha256 = compute_file_sha256(base_model_path)
    ds_sha256 = compute_file_sha256(dataset_path)

    hyperparams: Dict[str, Any] = {}
    if training_config is not None:
        hyperparams["training"] = {
            "learning_rate": training_config.learning_rate,
            "weight_decay": training_config.weight_decay,
            "batch_size": training_config.batch_size,
            "gradient_accumulation_steps": training_config.gradient_accumulation_steps,
            "max_epochs": training_config.max_epochs,
            "max_steps": training_config.max_steps,
            "warmup_steps": training_config.warmup_steps,
        }
    if lora_config is not None:
        hyperparams["lora"] = {
            "r": lora_config.r,
            "lora_alpha": lora_config.lora_alpha,
            "lora_dropout": lora_config.lora_dropout,
            "target_modules": lora_config.target_modules,
            "bias": lora_config.bias,
        }

    return TrainingJobProvenance(
        job_id=job_id,
        method=method,
        base_model_id=base_model_id,
        base_model_sha256=base_sha256,
        dataset_sha256=ds_sha256,
        seed=seed,
        device=device,
        metrics=metrics,
        hyperparameters=hyperparams,
    )


def export_adapter_package(
    output_dir: Union[str, Path],
    adapter_state_dict: Dict[str, torch.Tensor],
    provenance: TrainingJobProvenance,
    filename: str = "adapter_model.pt",
) -> Tuple[Path, Path]:
    """Exports standalone LoRA adapter weights and writes `checkpoint_provenance.json`."""
    out_dir = Path(output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    adapter_path = out_dir / filename
    tmp_path = adapter_path.with_suffix(".tmp")
    torch.save(adapter_state_dict, tmp_path)
    tmp_path.replace(adapter_path)

    # Compute digest and size
    adapter_sha256 = compute_file_sha256(adapter_path)
    adapter_size = adapter_path.stat().st_size

    provenance.adapter_sha256 = adapter_sha256
    provenance.adapter_size_bytes = adapter_size

    provenance_path = out_dir / "checkpoint_provenance.json"
    provenance.save_to_file(provenance_path)

    return adapter_path, provenance_path


def export_merged_model(
    model: nn.Module,
    output_path: Union[str, Path],
) -> Tuple[Path, str, int]:
    """Exports full merged model weights and computes its cryptographic SHA-256 digest."""
    target_path = Path(output_path)
    target_path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = target_path.with_suffix(".tmp")

    torch.save(model.state_dict(), tmp_path)
    tmp_path.replace(target_path)

    digest = compute_file_sha256(target_path)
    size_bytes = target_path.stat().st_size
    return target_path, digest, size_bytes


def create_registry_manifest(
    model_id: str,
    name: str,
    model_file_path: Union[str, Path],
    modality: Modality = Modality.LLM,
    version: str = "1.0.0",
    description: str = "MiniAI Fine-Tuned Model",
    format: str = "pt",
    parameter_count: Optional[str] = None,
    quantization: Optional[str] = "f32",
    min_ram_bytes: int = 1024 * 1024 * 512,
    recommended_backend: AcceleratorBackend = AcceleratorBackend.CPU,
    source_url: Optional[str] = None,
) -> Dict[str, Any]:
    """Constructs a `ModelManifest` payload compatible with MiniAI's Stage 2 Model Registry."""
    file_path = Path(model_file_path)
    if not file_path.exists():
        raise FileNotFoundError(f"Model file not found at: {file_path}")

    file_sha256 = compute_file_sha256(file_path)
    file_size = file_path.stat().st_size

    trimmed_id = model_id.strip()
    validate_safe_relative_path(trimmed_id)

    manifest = {
        "model_id": trimmed_id,
        "name": name.strip(),
        "modality": modality.value,
        "version": version.strip(),
        "description": description.strip(),
        "source_url": source_url or f"local://{file_path.name}",
        "expected_sha256": file_sha256,
        "expected_size_bytes": file_size,
        "format": format.strip(),
        "parameter_count": parameter_count,
        "quantization": quantization,
        "min_ram_bytes": min_ram_bytes,
        "recommended_backend": recommended_backend.value,
    }

    # Verify manifest integrity format
    if len(manifest["expected_sha256"]) != 64:
        raise ValueError("Invalid expected_sha256 generated")

    return manifest


def save_registry_manifest(
    manifest: Dict[str, Any],
    output_path: Union[str, Path],
) -> Path:
    """Saves registry manifest JSON to disk."""
    out_path = Path(output_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = out_path.with_suffix(".tmp")
    tmp_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    tmp_path.replace(out_path)
    return out_path


def verify_checkpoint_provenance(
    checkpoint_dir: Union[str, Path],
    base_model_path: Optional[Union[str, Path]] = None,
    dataset_path: Optional[Union[str, Path]] = None,
) -> TrainingJobProvenance:
    """Loads and verifies `checkpoint_provenance.json` in a checkpoint directory."""
    chk_dir = Path(checkpoint_dir)
    manifest_path = chk_dir / "checkpoint_provenance.json"
    if not manifest_path.exists():
        raise FileNotFoundError(
            f"checkpoint_provenance.json not found in directory: {chk_dir}"
        )

    provenance = TrainingJobProvenance.from_file(manifest_path)
    adapter_file = chk_dir / "adapter_model.pt"
    actual_adapter_path = adapter_file if adapter_file.exists() else None

    provenance.verify_integrity(
        base_model_path=base_model_path,
        dataset_path=dataset_path,
        adapter_path=actual_adapter_path,
    )
    return provenance
