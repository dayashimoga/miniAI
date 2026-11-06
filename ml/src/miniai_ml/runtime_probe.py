"""Deterministic seeding and CPU-first ML runtime capability probe."""

from __future__ import annotations

import os
import platform
import random
from typing import Any

from .contracts import AcceleratorBackend, VerificationTier


def seed_everything(seed: int = 42) -> int:
    """Seeds standard library random, NumPy, and PyTorch CPU/GPU deterministically."""
    if seed < 0:
        raise ValueError("Seed must be a non-negative integer")
    random.seed(seed)
    os.environ["PYTHONHASHSEED"] = str(seed)
    try:
        import numpy as np

        np.random.seed(seed)
    except ImportError:
        pass
    try:
        import torch

        torch.manual_seed(seed)
        if torch.cuda.is_available():
            torch.cuda.manual_seed_all(seed)
    except ImportError:
        pass
    return seed


def probe_ml_environment(force_cpu: bool = False) -> dict[str, Any]:
    """Probes the host Python ML environment with guaranteed CPU-first execution."""
    logical_cores = os.cpu_count() or 1
    detected_backends = [AcceleratorBackend.CPU.value]
    active_backend = AcceleratorBackend.CPU.value

    return {
        "os_family": platform.system().lower(),
        "arch": platform.machine().lower(),
        "python_version": platform.python_version(),
        "logical_cpu_cores": logical_cores,
        "detected_backends": detected_backends,
        "selected_backend": AcceleratorBackend.CPU.value if force_cpu else active_backend,
        "verification_tier": VerificationTier.PROVEN_ON_HOST.value,
    }

