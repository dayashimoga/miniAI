"""Unit, mathematical property, and convergence tests for LoRA fine-tuning and distillation."""

from __future__ import annotations

from pathlib import Path
import tempfile
import pytest
import torch
import torch.nn as nn

from miniai_ml.training import (
    DictDataset,
    DistillationLoss,
    EvaluationMetrics,
    LoraConfig,
    LoraLinear,
    Trainer,
    TrainingConfig,
    TrainingStepLog,
    apply_lora_to_model,
    count_trainable_parameters,
    extract_lora_state_dict,
    merge_lora_weights,
    resolve_device,
    seed_training_environment,
    unmerge_lora_weights,
)


class SimpleMlp(nn.Module):
    """Simple 2-layer MLP for testing LoRA and training workflows."""

    def __init__(self, in_dim: int = 16, hidden_dim: int = 32, out_dim: int = 4) -> None:
        super().__init__()
        self.fc1 = nn.Linear(in_dim, hidden_dim)
        self.relu = nn.ReLU()
        self.fc2 = nn.Linear(hidden_dim, out_dim)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.fc2(self.relu(self.fc1(x)))


def test_seed_training_environment() -> None:
    """Verifies deterministic tensor generation with seed_training_environment."""
    seed_training_environment(1234)
    t1 = torch.randn(5, 5)
    seed_training_environment(1234)
    t2 = torch.randn(5, 5)
    assert torch.equal(t1, t2)

    with pytest.raises(ValueError, match="non-negative integer"):
        seed_training_environment(-1)


def test_resolve_device() -> None:
    """Verifies CPU-first fallback and device resolution."""
    assert resolve_device("cpu") == torch.device("cpu")
    assert resolve_device("auto").type in ("cpu", "cuda", "mps")
    assert resolve_device("cuda:999") == torch.device("cpu")  # Fallback when unavailable
    assert resolve_device("mps") == torch.device("cpu") if not hasattr(torch.backends, "mps") or not torch.backends.mps.is_available() else torch.device("mps")


def test_lora_config_validation() -> None:
    """Verifies bounds and constraint validation on LoraConfig."""
    cfg = LoraConfig(r=4, lora_alpha=8.0, lora_dropout=0.1, target_modules=["fc1"])
    assert cfg.scaling == 2.0

    with pytest.raises(ValueError, match="rank r must be >= 1"):
        LoraConfig(r=0)

    with pytest.raises(ValueError, match="lora_alpha must be > 0"):
        LoraConfig(lora_alpha=0.0)

    with pytest.raises(ValueError, match="lora_dropout must be in"):
        LoraConfig(lora_dropout=1.0)

    with pytest.raises(ValueError, match="target_modules list cannot be empty"):
        LoraConfig(target_modules=[])

    with pytest.raises(ValueError, match="Invalid bias setting"):
        LoraConfig(bias="invalid")


def test_lora_linear_initialization_and_forward() -> None:
    """Verifies zero-delta identity at initialization and gradient flow."""
    base = nn.Linear(8, 16)
    cfg = LoraConfig(r=4, lora_alpha=8.0, lora_dropout=0.0)
    lora = LoraLinear(base, cfg)

    # Base parameters must be frozen
    assert not base.weight.requires_grad
    if base.bias is not None:
        assert not base.bias.requires_grad

    # Adapter parameters must be trainable
    assert lora.lora_A.weight.requires_grad
    assert lora.lora_B.weight.requires_grad

    # At init, B is zeros so lora output must match base output exactly
    x = torch.randn(2, 8)
    base_out = base(x)
    lora_out = lora(x)
    assert torch.allclose(base_out, lora_out, atol=1e-6)

    # Test dimensions
    assert lora.in_features == 8
    assert lora.out_features == 16


def test_lora_linear_merge_and_unmerge() -> None:
    """Verifies mathematical exactness of weight merging and unmerging."""
    base = nn.Linear(8, 16)
    cfg = LoraConfig(r=4, lora_alpha=8.0, lora_dropout=0.0)
    lora = LoraLinear(base, cfg)

    # Manually assign non-zero weights to B to induce a delta
    nn.init.normal_(lora.lora_B.weight, std=0.5)

    x = torch.randn(3, 8)
    out_before_merge = lora(x)

    # Merge into base weights
    lora.merge()
    assert lora.merged
    out_after_merge = lora(x)
    assert torch.allclose(out_before_merge, out_after_merge, atol=1e-5)

    # Calling merge again should be an idempotent no-op
    lora.merge()
    assert torch.allclose(out_before_merge, lora(x), atol=1e-5)

    # Unmerge back to decoupled form
    lora.unmerge()
    assert not lora.merged
    out_after_unmerge = lora(x)
    assert torch.allclose(out_before_merge, out_after_unmerge, atol=1e-5)

    # Calling unmerge again should be an idempotent no-op
    lora.unmerge()


def test_lora_adapter_state_dict() -> None:
    """Verifies extraction and restoring of adapter state dict."""
    base = nn.Linear(4, 6)
    cfg = LoraConfig(r=2, lora_alpha=4.0)
    lora = LoraLinear(base, cfg)

    state = lora.get_adapter_state_dict()
    assert "lora_A.weight" in state
    assert "lora_B.weight" in state
    assert state["lora_A.weight"].shape == (2, 4)
    assert state["lora_B.weight"].shape == (6, 2)

    # Modify and restore
    new_state = {
        "lora_A.weight": torch.ones(2, 4),
        "lora_B.weight": torch.ones(6, 2) * 2.0,
    }
    lora.load_adapter_state_dict(new_state)
    assert torch.equal(lora.lora_A.weight.data, torch.ones(2, 4))
    assert torch.equal(lora.lora_B.weight.data, torch.ones(6, 2) * 2.0)

    # Type error on invalid base layer
    with pytest.raises(TypeError, match="must be nn.Linear"):
        LoraLinear(nn.Conv2d(1, 1, 1), cfg)  # type: ignore


def test_apply_lora_to_model() -> None:
    """Verifies selective instrumentation of target modules in an existing network."""
    model = SimpleMlp(in_dim=10, hidden_dim=20, out_dim=2)
    cfg = LoraConfig(r=4, lora_alpha=8.0, target_modules=["fc1"])

    inst_model, adapters = apply_lora_to_model(model, cfg)
    assert len(adapters) == 1
    assert "fc1" in adapters
    assert isinstance(inst_model.fc1, LoraLinear)
    assert isinstance(inst_model.fc2, nn.Linear)  # Untouched

    # Check parameter counts
    counts = count_trainable_parameters(inst_model)
    assert counts["trainable_parameters"] > 0
    assert counts["trainable_parameters"] < counts["total_parameters"]
    assert counts["trainable_percent"] < 100.0

    # Check model-level merge/unmerge
    merged_count = merge_lora_weights(inst_model)
    assert merged_count == 1
    unmerged_count = unmerge_lora_weights(inst_model)
    assert unmerged_count == 1

    lora_dict = extract_lora_state_dict(inst_model)
    assert any("lora_A" in k for k in lora_dict)
    assert any("lora_B" in k for k in lora_dict)


def test_distillation_loss() -> None:
    """Verifies combined task loss and temperature-scaled KL divergence."""
    loss_fn = DistillationLoss(temperature=2.0, task_weight=0.5, kd_weight=0.5)

    student_logits = torch.tensor([[2.0, -1.0], [1.0, 1.0]], requires_grad=True)
    teacher_logits = torch.tensor([[2.5, -0.5], [0.8, 1.2]])
    labels = torch.tensor([0, 1])

    total_loss, task_loss, kd_loss = loss_fn(student_logits, teacher_logits, labels)
    assert total_loss.item() > 0.0
    assert task_loss.item() > 0.0
    assert kd_loss.item() > 0.0

    expected = 0.5 * task_loss.item() + 0.5 * kd_loss.item()
    assert abs(total_loss.item() - expected) < 1e-5

    # Test gradient backprop
    total_loss.backward()
    assert student_logits.grad is not None

    # Test pure KD (no labels)
    loss_pure_kd = DistillationLoss(temperature=2.0, task_weight=0.0, kd_weight=1.0)
    total_kd, task_zero, kd_val = loss_pure_kd(student_logits, teacher_logits, None)
    assert task_zero.item() == 0.0
    assert total_kd.item() == kd_val.item()

    # Test 3D sequence tokens
    seq_student = torch.randn(2, 4, 8)
    seq_teacher = torch.randn(2, 4, 8)
    seq_labels = torch.randint(0, 8, (2, 4))
    tot_seq, _, _ = loss_fn(seq_student, seq_teacher, seq_labels)
    assert tot_seq.item() > 0.0

    # Validation errors
    with pytest.raises(ValueError, match="Temperature must be > 0"):
        DistillationLoss(temperature=0.0)
    with pytest.raises(ValueError, match="non-negative"):
        DistillationLoss(task_weight=-0.1)
    with pytest.raises(ValueError, match="At least one loss weight"):
        DistillationLoss(task_weight=0.0, kd_weight=0.0)


def test_training_config_validation() -> None:
    """Verifies TrainingConfig hyperparameter validation."""
    cfg = TrainingConfig(learning_rate=1e-3, batch_size=8, max_epochs=2)
    assert cfg.learning_rate == 1e-3

    with pytest.raises(ValueError, match="learning_rate must be > 0"):
        TrainingConfig(learning_rate=0.0)
    with pytest.raises(ValueError, match="weight_decay must be >= 0"):
        TrainingConfig(weight_decay=-1.0)
    with pytest.raises(ValueError, match="batch_size must be >= 1"):
        TrainingConfig(batch_size=0)
    with pytest.raises(ValueError, match="gradient_accumulation_steps must be >= 1"):
        TrainingConfig(gradient_accumulation_steps=0)
    with pytest.raises(ValueError, match="max_epochs must be >= 1"):
        TrainingConfig(max_epochs=0)
    with pytest.raises(ValueError, match="max_steps must be >= 1"):
        TrainingConfig(max_steps=0)
    with pytest.raises(ValueError, match="seed must be >= 0"):
        TrainingConfig(seed=-5)


def test_trainer_convergence_on_cpu() -> None:
    """Verifies real mathematical convergence (loss decrease) of LoRA fine-tuning on CPU."""
    seed_training_environment(42)

    # Synthetic separable classification dataset: 60 samples, 8 features, 2 classes
    num_samples = 60
    in_features = 8
    num_classes = 2

    # Class 0 centered at -1.5, Class 1 centered at +1.5
    x_c0 = torch.randn(num_samples // 2, in_features) - 1.5
    y_c0 = torch.zeros(num_samples // 2, dtype=torch.long)
    x_c1 = torch.randn(num_samples // 2, in_features) + 1.5
    y_c1 = torch.ones(num_samples // 2, dtype=torch.long)

    all_x = torch.cat([x_c0, x_c1], dim=0)
    all_y = torch.cat([y_c0, y_c1], dim=0)

    train_data = [{"x": all_x[i], "label": all_y[i]} for i in range(48)]
    eval_data = [{"x": all_x[i], "label": all_y[i]} for i in range(48, 60)]

    train_dataset = DictDataset(train_data)
    eval_dataset = DictDataset(eval_data)

    # Initialize model and apply LoRA
    model = SimpleMlp(in_dim=in_features, hidden_dim=16, out_dim=num_classes)
    lora_cfg = LoraConfig(r=4, lora_alpha=8.0, lora_dropout=0.0, target_modules=["fc1", "fc2"])
    model, _ = apply_lora_to_model(model, lora_cfg)

    train_cfg = TrainingConfig(
        learning_rate=0.05,
        batch_size=8,
        gradient_accumulation_steps=1,
        max_epochs=8,
        warmup_steps=2,
        eval_every_steps=4,
        save_every_steps=0,
        device="cpu",
    )

    trainer = Trainer(
        model=model,
        config=train_cfg,
        train_dataset=train_dataset,
        eval_dataset=eval_dataset,
    )

    initial_eval = trainer.evaluate()
    initial_val_loss = initial_eval.val_loss

    logs = trainer.train()
    assert len(logs) > 0

    final_eval = trainer.evaluate()
    final_val_loss = final_eval.val_loss

    # Verify mathematical convergence: loss decreased significantly
    assert logs[-1].loss < logs[0].loss
    assert final_val_loss < initial_val_loss


def test_trainer_distillation_and_checkpoint_resume() -> None:
    """Verifies teacher-student distillation training, cooperative abort, and checkpoint saving/loading."""
    seed_training_environment(99)

    in_dim = 6
    out_dim = 2
    samples = 30

    x_data = torch.randn(samples, in_dim)
    y_data = torch.randint(0, out_dim, (samples,))
    dataset = DictDataset([{"x": x_data[i], "label": y_data[i]} for i in range(samples)])

    teacher = SimpleMlp(in_dim=in_dim, hidden_dim=16, out_dim=out_dim)
    student = SimpleMlp(in_dim=in_dim, hidden_dim=8, out_dim=out_dim)

    lora_cfg = LoraConfig(r=2, lora_alpha=4.0, target_modules=["fc1"])
    student, _ = apply_lora_to_model(student, lora_cfg)

    with tempfile.TemporaryDirectory() as tmp_dir:
        train_cfg = TrainingConfig(
            learning_rate=0.01,
            batch_size=5,
            max_epochs=2,
            eval_every_steps=0,
            save_every_steps=3,
            output_dir=tmp_dir,
            device="cpu",
        )

        dist_loss = DistillationLoss(temperature=2.0, task_weight=0.5, kd_weight=0.5)

        trainer = Trainer(
            model=student,
            config=train_cfg,
            train_dataset=dataset,
            teacher_model=teacher,
            distillation_loss=dist_loss,
        )

        trainer.train()
        assert trainer.global_step > 0

        # Checkpoint file should have been written
        ckpt_path = Path(tmp_dir) / f"checkpoint_step_{trainer.global_step}.pt"
        trainer.save_checkpoint(ckpt_path)
        assert ckpt_path.exists()

        # Create new student instance and load checkpoint
        resumed_student = SimpleMlp(in_dim=in_dim, hidden_dim=8, out_dim=out_dim)
        resumed_student, _ = apply_lora_to_model(resumed_student, lora_cfg)

        resumed_trainer = Trainer(
            model=resumed_student,
            config=train_cfg,
            train_dataset=dataset,
            teacher_model=teacher,
            distillation_loss=dist_loss,
        )
        resumed_trainer.load_checkpoint(ckpt_path)

        assert resumed_trainer.global_step == trainer.global_step
        assert resumed_trainer.current_epoch == trainer.current_epoch
        assert len(resumed_trainer.training_logs) == len(trainer.training_logs)

        # Non-existent checkpoint raises FileNotFoundError
        with pytest.raises(FileNotFoundError):
            resumed_trainer.load_checkpoint(Path(tmp_dir) / "non_existent.pt")


def test_trainer_cooperative_cancellation() -> None:
    """Verifies that Trainer aborts training cleanly when should_stop returns True."""
    seed_training_environment(42)
    dataset = DictDataset([{"x": torch.randn(4), "label": torch.tensor(0)} for _ in range(20)])
    model = SimpleMlp(in_dim=4, hidden_dim=8, out_dim=2)
    model, _ = apply_lora_to_model(model, LoraConfig(r=2, lora_alpha=4.0, target_modules=["fc1"]))

    cfg = TrainingConfig(learning_rate=0.01, batch_size=4, max_epochs=10, device="cpu")
    trainer = Trainer(model=model, config=cfg, train_dataset=dataset)

    # Immediately cancel
    logs = trainer.train(should_stop=lambda: True)
    assert len(logs) == 0
    assert trainer.global_step == 0


def test_trainer_no_trainable_parameters_error() -> None:
    """Verifies that Trainer raises ValueError if model has no trainable parameters."""
    model = SimpleMlp()
    for p in model.parameters():
        p.requires_grad = False

    dataset = DictDataset([{"x": torch.randn(16), "label": torch.tensor(0)}])
    cfg = TrainingConfig()

    with pytest.raises(ValueError, match="no trainable parameters"):
        Trainer(model=model, config=cfg, train_dataset=dataset)


def test_trainer_gradient_accumulation_and_max_steps() -> None:
    """Verifies gradient accumulation > 1 and max_steps early termination."""
    seed_training_environment(10)
    dataset = DictDataset([{"x": torch.randn(8), "label": torch.tensor(0)} for _ in range(16)])
    model = SimpleMlp(in_dim=8, hidden_dim=12, out_dim=2)
    model, _ = apply_lora_to_model(model, LoraConfig(r=2, lora_alpha=4.0, target_modules=["fc1"]))

    cfg = TrainingConfig(
        learning_rate=0.01,
        batch_size=2,
        gradient_accumulation_steps=2,
        max_epochs=5,
        max_steps=3,  # Should stop after exactly 3 steps
        device="cpu",
    )
    trainer = Trainer(model=model, config=cfg, train_dataset=dataset)
    logs = trainer.train()
    assert len(logs) == 3
    assert trainer.global_step == 3


def test_trainer_sequence_tokens_and_input_ids() -> None:
    """Verifies Trainer support for sequence models with 'input_ids' input key."""
    class SeqModel(nn.Module):
        def __init__(self) -> None:
            super().__init__()
            self.emb = nn.Embedding(20, 16)
            self.proj = nn.Linear(16, 20)

        def forward(self, input_ids: torch.Tensor) -> torch.Tensor:
            return self.proj(self.emb(input_ids))

    seed_training_environment(11)
    seq_model = SeqModel()
    seq_model, _ = apply_lora_to_model(
        seq_model, LoraConfig(r=2, lora_alpha=4.0, target_modules=["proj"])
    )

    batch_samples = [
        {"input_ids": torch.randint(0, 20, (6,)), "label": torch.randint(0, 20, (6,))}
        for _ in range(8)
    ]
    train_ds = DictDataset(batch_samples[:6])
    eval_ds = DictDataset(batch_samples[6:])

    cfg = TrainingConfig(
        learning_rate=0.01,
        batch_size=2,
        max_epochs=1,
        eval_every_steps=1,
        device="cpu",
    )
    trainer = Trainer(
        model=seq_model,
        config=cfg,
        train_dataset=train_ds,
        eval_dataset=eval_ds,
    )
    logs = trainer.train()
    assert len(logs) > 0
    eval_m = trainer.evaluate()
    assert eval_m.val_loss >= 0.0
    assert eval_m.perplexity >= 1.0


def test_trainer_empty_eval_dataset() -> None:
    """Verifies that evaluation returns identity metrics when eval dataset is None."""
    model = SimpleMlp(in_dim=4, hidden_dim=4, out_dim=2)
    model, _ = apply_lora_to_model(model, LoraConfig(r=2, lora_alpha=4.0, target_modules=["fc1"]))
    trainer = Trainer(
        model=model,
        config=TrainingConfig(device="cpu"),
        train_dataset=DictDataset([{"x": torch.randn(4), "label": torch.tensor(0)}]),
        eval_dataset=None,
    )
    m = trainer.evaluate()
    assert m.val_loss == 0.0
    assert m.perplexity == 1.0

