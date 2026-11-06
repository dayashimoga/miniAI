"""MiniAI LoRA/QLoRA Parameter-Efficient Fine-Tuning & Knowledge Distillation Engine.

This module provides CPU-first, mathematically verified implementations of:
1. Low-Rank Adaptation (LoRA) layers (`LoraLinear`) and model instrumentation.
2. Hinton-style teacher-student knowledge distillation loss (`DistillationLoss`).
3. Training loop orchestration (`Trainer`), deterministic seeding, and checkpointing.
"""

from __future__ import annotations

import copy
from dataclasses import asdict, dataclass, field
import json
import math
from pathlib import Path
import random
from typing import Any, Callable, Dict, Iterator, List, Optional, Sequence, Tuple, Union

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.optim import AdamW
from torch.utils.data import DataLoader, Dataset


def seed_training_environment(seed: int = 42) -> int:
    """Sets deterministic random seeds across Python, NumPy, and PyTorch CPU/GPU.

    Args:
        seed: Non-negative random seed.

    Returns:
        The seed integer set.
    """
    if seed < 0:
        raise ValueError(f"Seed must be a non-negative integer, got {seed}")

    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)
    if torch.cuda.is_available():
        torch.cuda.manual_seed_all(seed)
        torch.backends.cudnn.deterministic = True
        torch.backends.cudnn.benchmark = False
    return seed


def resolve_device(requested: Optional[str] = None) -> torch.device:
    """Resolves an execution device prioritizing CPU baseline when hardware is unverified.

    Args:
        requested: Requested device string ("cpu", "cuda", "mps", or None for auto).

    Returns:
        Resolved `torch.device`.
    """
    if requested is None or requested.lower() == "auto":
        if torch.cuda.is_available():
            return torch.device("cuda")
        if hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
            return torch.device("mps")
        return torch.device("cpu")

    dev_str = requested.lower()
    if dev_str.startswith("cuda"):
        if torch.cuda.is_available():
            return torch.device(dev_str)
        return torch.device("cpu")
    if dev_str == "mps":
        if hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
            return torch.device("mps")
        return torch.device("cpu")
    return torch.device("cpu")


@dataclass(frozen=True)
class LoraConfig:
    """Configuration for Low-Rank Adaptation (LoRA).

    Attributes:
        r: Rank of the low-rank update decomposition matrices (A and B). Must be >= 1.
        lora_alpha: Scaling factor constant. Must be > 0.
        lora_dropout: Dropout probability applied to input before low-rank projection.
        target_modules: Names or sub-tokens of linear layers to instrument.
        bias: Bias handling strategy ("none", "all", or "lora_only").
    """

    r: int = 8
    lora_alpha: float = 16.0
    lora_dropout: float = 0.05
    target_modules: List[str] = field(default_factory=lambda: ["q_proj", "v_proj"])
    bias: str = "none"

    def __post_init__(self) -> None:
        if self.r < 1:
            raise ValueError(f"LoRA rank r must be >= 1, got {self.r}")
        if self.lora_alpha <= 0.0:
            raise ValueError(f"lora_alpha must be > 0, got {self.lora_alpha}")
        if not (0.0 <= self.lora_dropout < 1.0):
            raise ValueError(
                f"lora_dropout must be in [0.0, 1.0), got {self.lora_dropout}"
            )
        if not self.target_modules:
            raise ValueError("target_modules list cannot be empty")
        if self.bias not in ("none", "all", "lora_only"):
            raise ValueError(
                f"Invalid bias setting: {self.bias}. Must be one of: none, all, lora_only"
            )

    @property
    def scaling(self) -> float:
        """The constant scaling factor alpha / r."""
        return self.lora_alpha / float(self.r)


class LoraLinear(nn.Module):
    """Low-Rank Adaptation linear layer wrapping a frozen base `nn.Linear`.

    Given base weight W_0 and input x:
        h = W_0 * x + (alpha / r) * (B * A * x)
    where A is initialized with Kaiming uniform and B is initialized to zeros,
    guaranteeing h = W_0 * x at training start.
    """

    def __init__(self, base_layer: nn.Linear, config: LoraConfig) -> None:
        super().__init__()
        if not isinstance(base_layer, nn.Linear):
            raise TypeError(f"base_layer must be nn.Linear, got {type(base_layer)}")

        self.base_layer = base_layer
        self.config = config
        self.r = config.r
        self.scaling = config.scaling
        self.merged = False

        in_features = base_layer.in_features
        out_features = base_layer.out_features

        # Freeze base parameters
        for param in self.base_layer.parameters():
            param.requires_grad = False

        # Initialize low-rank adapter projections
        self.lora_dropout = (
            nn.Dropout(p=config.lora_dropout)
            if config.lora_dropout > 0.0
            else nn.Identity()
        )
        self.lora_A = nn.Linear(in_features, self.r, bias=False)
        self.lora_B = nn.Linear(self.r, out_features, bias=False)

        # Mathematical initialization: A ~ Kaiming uniform, B = 0
        nn.init.kaiming_uniform_(self.lora_A.weight, a=math.sqrt(5))
        nn.init.zeros_(self.lora_B.weight)

    @property
    def in_features(self) -> int:
        return self.base_layer.in_features

    @property
    def out_features(self) -> int:
        return self.base_layer.out_features

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """Forward pass combining base projection and low-rank delta."""
        base_out = self.base_layer(x)
        if self.merged:
            return base_out

        # Compute delta = (alpha / r) * B(A(dropout(x)))
        dropped = self.lora_dropout(x)
        projected_a = self.lora_A(dropped)
        projected_b = self.lora_B(projected_a)
        return base_out + self.scaling * projected_b

    def merge(self) -> None:
        """Merges low-rank adapter weights directly into base weight matrix."""
        if self.merged:
            return

        # Delta W = scaling * (B @ A)
        delta_w = self.scaling * (self.lora_B.weight.data @ self.lora_A.weight.data)
        self.base_layer.weight.data.add_(delta_w)
        self.merged = True

    def unmerge(self) -> None:
        """Subtracts low-rank adapter weights from base weight matrix."""
        if not self.merged:
            return

        delta_w = self.scaling * (self.lora_B.weight.data @ self.lora_A.weight.data)
        self.base_layer.weight.data.sub_(delta_w)
        self.merged = False

    def get_adapter_state_dict(self) -> Dict[str, torch.Tensor]:
        """Returns detached copy of the trainable adapter weights."""
        return {
            "lora_A.weight": self.lora_A.weight.data.clone().detach(),
            "lora_B.weight": self.lora_B.weight.data.clone().detach(),
        }

    def load_adapter_state_dict(self, state_dict: Dict[str, torch.Tensor]) -> None:
        """Loads weights into adapter projections."""
        if "lora_A.weight" in state_dict:
            self.lora_A.weight.data.copy_(state_dict["lora_A.weight"])
        if "lora_B.weight" in state_dict:
            self.lora_B.weight.data.copy_(state_dict["lora_B.weight"])


def apply_lora_to_model(
    model: nn.Module, config: LoraConfig
) -> Tuple[nn.Module, Dict[str, LoraLinear]]:
    """Recursively replaces targeted `nn.Linear` layers with `LoraLinear` adapters.

    Args:
        model: Base PyTorch neural network module.
        config: LoRA configuration parameters.

    Returns:
        Tuple of (instrumented model, dictionary of module path -> LoraLinear adapter).
    """
    adapters: Dict[str, LoraLinear] = {}

    def _replace_modules(parent: nn.Module, parent_prefix: str = "") -> None:
        for name, child in list(parent.named_children()):
            full_path = f"{parent_prefix}.{name}" if parent_prefix else name
            matches = any(target in full_path or target in name for target in config.target_modules)

            if isinstance(child, nn.Linear) and matches:
                lora_layer = LoraLinear(child, config)
                setattr(parent, name, lora_layer)
                adapters[full_path] = lora_layer
            else:
                _replace_modules(child, full_path)

    _replace_modules(model)

    # Freeze any remaining parameters that are not LoRA adapters
    for name, param in model.named_parameters():
        if "lora_" not in name:
            param.requires_grad = False
        else:
            param.requires_grad = True

    return model, adapters


def merge_lora_weights(model: nn.Module) -> int:
    """Merges all `LoraLinear` adapters in a model into their base weights.

    Returns:
        Count of merged adapter layers.
    """
    count = 0
    for module in model.modules():
        if isinstance(module, LoraLinear):
            module.merge()
            count += 1
    return count


def unmerge_lora_weights(model: nn.Module) -> int:
    """Unmerges all `LoraLinear` adapters in a model back to factored form.

    Returns:
        Count of unmerged adapter layers.
    """
    count = 0
    for module in model.modules():
        if isinstance(module, LoraLinear):
            module.unmerge()
            count += 1
    return count


def extract_lora_state_dict(model: nn.Module) -> Dict[str, torch.Tensor]:
    """Extracts only trainable LoRA adapter parameters from a model."""
    return {
        name: param.data.clone().detach()
        for name, param in model.named_parameters()
        if "lora_" in name and param.requires_grad
    }


def count_trainable_parameters(model: nn.Module) -> Dict[str, Union[int, float]]:
    """Calculates total, trainable, and percentage trainable parameter counts."""
    total = sum(p.numel() for p in model.parameters())
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    pct = (float(trainable) / float(total) * 100.0) if total > 0 else 0.0
    return {
        "total_parameters": total,
        "trainable_parameters": trainable,
        "trainable_percent": round(pct, 4),
    }


class DistillationLoss(nn.Module):
    """Teacher-Student Knowledge Distillation Loss combining task loss and softened KL divergence.

    Formula:
        L_total = task_weight * L_task + kd_weight * (T^2) * D_KL(sigma(z_s / T) || sigma(z_t / T))
    """

    def __init__(
        self,
        temperature: float = 2.0,
        task_weight: float = 0.5,
        kd_weight: float = 0.5,
    ) -> None:
        super().__init__()
        if temperature <= 0.0:
            raise ValueError(f"Temperature must be > 0, got {temperature}")
        if task_weight < 0.0 or kd_weight < 0.0:
            raise ValueError("Loss weights must be non-negative")
        if task_weight == 0.0 and kd_weight == 0.0:
            raise ValueError("At least one loss weight must be positive")

        self.temperature = temperature
        self.task_weight = task_weight
        self.kd_weight = kd_weight
        self.kl_div = nn.KLDivLoss(reduction="batchmean")

    def forward(
        self,
        student_logits: torch.Tensor,
        teacher_logits: Optional[torch.Tensor] = None,
        labels: Optional[torch.Tensor] = None,
    ) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """Calculates combined distillation loss.

        Args:
            student_logits: Raw prediction tensor from student model [B, ..., C].
            teacher_logits: Optional prediction tensor from teacher model [B, ..., C].
            labels: Optional ground-truth target tensor.

        Returns:
            Tuple of (total_loss, task_loss, kd_loss).
        """
        device = student_logits.device
        zero = torch.tensor(0.0, device=device)
        task_loss = zero
        kd_loss = zero

        # 1. Compute ground-truth task loss if labels are provided
        if labels is not None and self.task_weight > 0.0:
            if student_logits.dim() == 2:
                task_loss = F.cross_entropy(student_logits, labels)
            else:
                # Handle sequence tokens [Batch, SeqLen, Vocab] -> flatten to [B*S, V]
                vocab_size = student_logits.size(-1)
                task_loss = F.cross_entropy(
                    student_logits.view(-1, vocab_size), labels.view(-1)
                )

        # 2. Compute temperature-scaled KL divergence if teacher logits are provided
        if teacher_logits is not None and self.kd_weight > 0.0:
            s_soft = F.log_softmax(student_logits / self.temperature, dim=-1)
            t_soft = F.softmax(teacher_logits / self.temperature, dim=-1)
            # Scaling by T^2 preserves gradient magnitude relative to task loss
            kd_loss = self.kl_div(s_soft, t_soft) * (self.temperature**2)

        # 3. Combine weighted losses
        total_loss = (self.task_weight * task_loss) + (self.kd_weight * kd_loss)
        return total_loss, task_loss, kd_loss


@dataclass
class TrainingConfig:
    """Hyperparameter and hardware configuration for training runs.

    Attributes:
        learning_rate: Initial optimizer learning rate.
        weight_decay: L2 regularization coefficient.
        batch_size: Mini-batch size.
        gradient_accumulation_steps: Number of steps before updating weights.
        max_epochs: Maximum complete epochs over training dataset.
        max_steps: Optional step limit overriding epochs.
        warmup_steps: Number of linear warmup steps.
        seed: Deterministic PRNG seed.
        device: Target execution device ("cpu", "cuda", "mps", "auto").
        eval_every_steps: Evaluation frequency in steps.
        save_every_steps: Checkpoint saving frequency in steps.
        clip_grad_norm: Optional maximum gradient norm threshold.
        output_dir: Directory where checkpoints and provenance logs are stored.
    """

    learning_rate: float = 1e-4
    weight_decay: float = 0.01
    batch_size: int = 4
    gradient_accumulation_steps: int = 1
    max_epochs: int = 3
    max_steps: Optional[int] = None
    warmup_steps: int = 0
    seed: int = 42
    device: str = "cpu"
    eval_every_steps: int = 10
    save_every_steps: int = 50
    clip_grad_norm: Optional[float] = 1.0
    output_dir: str = "checkpoints"

    def __post_init__(self) -> None:
        if self.learning_rate <= 0.0:
            raise ValueError(f"learning_rate must be > 0, got {self.learning_rate}")
        if self.weight_decay < 0.0:
            raise ValueError(f"weight_decay must be >= 0, got {self.weight_decay}")
        if self.batch_size < 1:
            raise ValueError(f"batch_size must be >= 1, got {self.batch_size}")
        if self.gradient_accumulation_steps < 1:
            raise ValueError(
                f"gradient_accumulation_steps must be >= 1, got {self.gradient_accumulation_steps}"
            )
        if self.max_epochs < 1:
            raise ValueError(f"max_epochs must be >= 1, got {self.max_epochs}")
        if self.max_steps is not None and self.max_steps < 1:
            raise ValueError(f"max_steps must be >= 1, got {self.max_steps}")
        if self.seed < 0:
            raise ValueError(f"seed must be >= 0, got {self.seed}")


@dataclass
class TrainingStepLog:
    """Telemetry recorded at each training optimization step."""

    step: int
    epoch: int
    loss: float
    task_loss: float
    kd_loss: float
    learning_rate: float
    grad_norm: float


@dataclass
class EvaluationMetrics:
    """Evaluation summary on validation split."""

    step: int
    epoch: int
    val_loss: float
    perplexity: float


class DictDataset(Dataset):
    """Simple in-memory PyTorch Dataset wrapping dictionaries or tuples of tensors."""

    def __init__(self, items: Sequence[Dict[str, torch.Tensor]]) -> None:
        self.items = list(items)

    def __len__(self) -> int:
        return len(self.items)

    def __getitem__(self, idx: int) -> Dict[str, torch.Tensor]:
        return self.items[idx]


def default_collate_dict(batch: List[Dict[str, torch.Tensor]]) -> Dict[str, torch.Tensor]:
    """Collate list of sample dicts into batched tensors."""
    elem = batch[0]
    collated: Dict[str, torch.Tensor] = {}
    for key in elem:
        tensors = [sample[key] for sample in batch]
        collated[key] = torch.stack(tensors, dim=0)
    return collated


class Trainer:
    """Orchestrates model training, teacher-student distillation, and checkpointing."""

    def __init__(
        self,
        model: nn.Module,
        config: TrainingConfig,
        train_dataset: Dataset,
        eval_dataset: Optional[Dataset] = None,
        teacher_model: Optional[nn.Module] = None,
        distillation_loss: Optional[DistillationLoss] = None,
        optimizer: Optional[torch.optim.Optimizer] = None,
        collate_fn: Optional[Callable] = None,
    ) -> None:
        self.config = config
        seed_training_environment(config.seed)
        self.device = resolve_device(config.device)

        self.model = model.to(self.device)
        self.teacher_model = (
            teacher_model.to(self.device) if teacher_model is not None else None
        )
        if self.teacher_model is not None:
            self.teacher_model.eval()
            for p in self.teacher_model.parameters():
                p.requires_grad = False

        self.distillation_loss = distillation_loss or DistillationLoss()
        self.train_dataset = train_dataset
        self.eval_dataset = eval_dataset
        self.collate_fn = collate_fn or default_collate_dict

        # Collect trainable parameters
        trainable_params = [p for p in self.model.parameters() if p.requires_grad]
        if not trainable_params:
            raise ValueError("Model has no trainable parameters")

        self.optimizer = optimizer or AdamW(
            trainable_params,
            lr=config.learning_rate,
            weight_decay=config.weight_decay,
        )

        self.global_step = 0
        self.current_epoch = 0
        self.training_logs: List[TrainingStepLog] = []
        self.eval_metrics: List[EvaluationMetrics] = []

    def _get_lr(self, step: int) -> float:
        """Computes learning rate with linear warmup if configured."""
        if self.config.warmup_steps <= 0:
            return self.config.learning_rate
        if step < self.config.warmup_steps:
            return self.config.learning_rate * float(step + 1) / float(self.config.warmup_steps)
        return self.config.learning_rate

    def evaluate(self) -> EvaluationMetrics:
        """Evaluates model performance across the evaluation dataset."""
        if self.eval_dataset is None or len(self.eval_dataset) == 0:
            return EvaluationMetrics(
                step=self.global_step,
                epoch=self.current_epoch,
                val_loss=0.0,
                perplexity=1.0,
            )

        self.model.eval()
        loader = DataLoader(
            self.eval_dataset,
            batch_size=self.config.batch_size,
            shuffle=False,
            collate_fn=self.collate_fn,
        )

        total_loss = 0.0
        total_samples = 0

        with torch.no_grad():
            for batch in loader:
                inputs = {
                    k: v.to(self.device)
                    for k, v in batch.items()
                    if k not in ("label", "labels", "target", "targets")
                }
                labels = batch.get("label", batch.get("labels", batch.get("target")))
                if labels is not None:
                    labels = labels.to(self.device)

                if "input_ids" in inputs:
                    student_out = self.model(inputs["input_ids"])
                elif "x" in inputs:
                    student_out = self.model(inputs["x"])
                else:
                    first_tensor = next(iter(inputs.values()))
                    student_out = self.model(first_tensor)

                if labels is not None:
                    if student_out.dim() == 2:
                        batch_loss = F.cross_entropy(student_out, labels)
                    else:
                        batch_loss = F.cross_entropy(
                            student_out.view(-1, student_out.size(-1)), labels.view(-1)
                        )
                else:
                    batch_loss = torch.tensor(0.0, device=self.device)

                bsize = len(next(iter(batch.values())))
                total_loss += float(batch_loss.item()) * bsize
                total_samples += bsize

        mean_val_loss = (total_loss / total_samples) if total_samples > 0 else 0.0
        ppl = math.exp(min(mean_val_loss, 20.0))
        metrics = EvaluationMetrics(
            step=self.global_step,
            epoch=self.current_epoch,
            val_loss=round(mean_val_loss, 4),
            perplexity=round(ppl, 4),
        )
        self.eval_metrics.append(metrics)
        return metrics

    def train(
        self,
        should_stop: Optional[Callable[[], bool]] = None,
    ) -> List[TrainingStepLog]:
        """Executes the training loop with gradient accumulation and evaluation.

        Args:
            should_stop: Optional cooperative cancellation callback returning True to abort.

        Returns:
            List of recorded `TrainingStepLog` entries.
        """
        train_loader = DataLoader(
            self.train_dataset,
            batch_size=self.config.batch_size,
            shuffle=True,
            collate_fn=self.collate_fn,
        )

        self.model.train()
        accum_steps = self.config.gradient_accumulation_steps
        self.optimizer.zero_grad()

        for epoch in range(self.current_epoch, self.config.max_epochs):
            self.current_epoch = epoch

            for batch_idx, batch in enumerate(train_loader):
                if should_stop is not None and should_stop():
                    return self.training_logs

                # Update learning rate
                current_lr = self._get_lr(self.global_step)
                for param_group in self.optimizer.param_groups:
                    param_group["lr"] = current_lr

                inputs = {
                    k: v.to(self.device)
                    for k, v in batch.items()
                    if k not in ("label", "labels", "target", "targets")
                }
                labels = batch.get("label", batch.get("labels", batch.get("target")))
                if labels is not None:
                    labels = labels.to(self.device)

                # Student forward pass
                if "input_ids" in inputs:
                    student_out = self.model(inputs["input_ids"])
                elif "x" in inputs:
                    student_out = self.model(inputs["x"])
                else:
                    first_tensor = next(iter(inputs.values()))
                    student_out = self.model(first_tensor)

                # Teacher forward pass (if present)
                teacher_out = None
                if self.teacher_model is not None:
                    with torch.no_grad():
                        if "input_ids" in inputs:
                            teacher_out = self.teacher_model(inputs["input_ids"])
                        elif "x" in inputs:
                            teacher_out = self.teacher_model(inputs["x"])
                        else:
                            first_tensor = next(iter(inputs.values()))
                            teacher_out = self.teacher_model(first_tensor)

                total_loss, task_loss, kd_loss = self.distillation_loss(
                    student_out, teacher_out, labels
                )

                # Normalize loss for gradient accumulation
                scaled_loss = total_loss / float(accum_steps)
                scaled_loss.backward()

                # Optimization step when accumulation boundary is reached
                if (batch_idx + 1) % accum_steps == 0 or (batch_idx + 1) == len(train_loader):
                    grad_norm = 0.0
                    if self.config.clip_grad_norm is not None:
                        grad_norm = float(
                            nn.utils.clip_grad_norm_(
                                [p for p in self.model.parameters() if p.requires_grad],
                                self.config.clip_grad_norm,
                            )
                        )

                    self.optimizer.step()
                    self.optimizer.zero_grad()
                    self.global_step += 1

                    log_entry = TrainingStepLog(
                        step=self.global_step,
                        epoch=epoch,
                        loss=round(float(total_loss.item()), 5),
                        task_loss=round(float(task_loss.item()), 5),
                        kd_loss=round(float(kd_loss.item()), 5),
                        learning_rate=current_lr,
                        grad_norm=round(grad_norm, 5),
                    )
                    self.training_logs.append(log_entry)

                    # Periodic evaluation
                    if (
                        self.config.eval_every_steps > 0
                        and self.global_step % self.config.eval_every_steps == 0
                    ):
                        self.evaluate()
                        self.model.train()

                    # Periodic checkpointing
                    if (
                        self.config.save_every_steps > 0
                        and self.global_step % self.config.save_every_steps == 0
                    ):
                        ckpt_path = (
                            Path(self.config.output_dir) / f"checkpoint_step_{self.global_step}.pt"
                        )
                        self.save_checkpoint(ckpt_path)

                    if self.config.max_steps and self.global_step >= self.config.max_steps:
                        return self.training_logs

        return self.training_logs

    def save_checkpoint(self, path: Union[str, Path]) -> Path:
        """Saves an atomic training checkpoint containing weights, optimizer state, and step."""
        save_path = Path(path)
        save_path.parent.mkdir(parents=True, exist_ok=True)
        tmp_path = save_path.with_suffix(".tmp")

        checkpoint = {
            "global_step": self.global_step,
            "current_epoch": self.current_epoch,
            "model_state_dict": self.model.state_dict(),
            "adapter_state_dict": extract_lora_state_dict(self.model),
            "optimizer_state_dict": self.optimizer.state_dict(),
            "config": asdict(self.config),
            "training_logs": [asdict(log) for log in self.training_logs],
            "eval_metrics": [asdict(m) for m in self.eval_metrics],
        }

        torch.save(checkpoint, tmp_path)
        tmp_path.replace(save_path)
        return save_path

    def load_checkpoint(self, path: Union[str, Path]) -> None:
        """Restores model, optimizer, step, and metric state from a saved checkpoint."""
        load_path = Path(path)
        if not load_path.exists():
            raise FileNotFoundError(f"Checkpoint not found at: {load_path}")

        checkpoint = torch.load(load_path, map_location=self.device)
        self.global_step = checkpoint["global_step"]
        self.current_epoch = checkpoint["current_epoch"]
        self.model.load_state_dict(checkpoint["model_state_dict"])
        self.optimizer.load_state_dict(checkpoint["optimizer_state_dict"])

        self.training_logs = [
            TrainingStepLog(**log) for log in checkpoint.get("training_logs", [])
        ]
        self.eval_metrics = [
            EvaluationMetrics(**m) for m in checkpoint.get("eval_metrics", [])
        ]
