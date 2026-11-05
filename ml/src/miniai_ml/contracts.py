"""Cross-language contracts, SHA-256 integrity checks, and path sanitization for MiniAI."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import hashlib
from typing import Any


class Modality(str, Enum):
    LLM = "llm"
    VISION = "vision"
    IMAGE_GENERATION = "image_generation"
    SPEECH_ASR = "speech_asr"
    SPEECH_TTS = "speech_tts"
    MUSIC = "music"
    VIDEO = "video"


class AcceleratorBackend(str, Enum):
    CPU = "cpu"
    CUDA = "cuda"
    VULKAN = "vulkan"
    METAL = "metal"
    DIRECT_ML = "direct_ml"
    ROCM = "rocm"
    CORE_ML = "core_ml"
    NNAPI = "nnapi"

    @property
    def is_cpu(self) -> bool:
        return self == AcceleratorBackend.CPU


class VerificationTier(str, Enum):
    PROVEN_ON_HOST = "PROVEN_ON_HOST"
    VERIFIED_IN_CI = "VERIFIED_IN_CI"
    HARDWARE_DEPENDENT = "HARDWARE_DEPENDENT"


def compute_sha256_hex(data: bytes) -> str:
    """Computes lowercase 64-character hexadecimal SHA-256 digest."""
    return hashlib.sha256(data).hexdigest().lower()


def verify_sha256_digest(data: bytes, expected_hex: str) -> str:
    """Verifies that `data` matches `expected_hex`, raising ValueError on mismatch or bad format."""
    normalized = expected_hex.strip().lower()
    if len(normalized) != 64 or any(c not in "0123456789abcdef" for c in normalized):
        raise ValueError(f"Invalid SHA-256 hex digest format: {expected_hex!r}")
    actual = compute_sha256_hex(data)
    if actual != normalized:
        raise ValueError(f"SHA-256 integrity mismatch: expected {normalized}, got {actual}")
    return actual


def validate_safe_relative_path(path_str: str) -> str:
    """Validates that a relative path contains no traversal, root slashes, or drive letters."""
    trimmed = path_str.strip()
    if (
        not trimmed
        or "\x00" in trimmed
        or trimmed.startswith("/")
        or trimmed.startswith("\\")
        or ":" in trimmed
    ):
        raise ValueError(f"Unsafe path rejected: {path_str!r}")

    normalized = trimmed.replace("\\", "/")
    segments = normalized.split("/")
    for seg in segments:
        if not seg or seg in (".", ".."):
            raise ValueError(f"Unsafe path segment rejected in: {path_str!r}")
    return normalized


@dataclass(frozen=True)
class HardwareProfile:
    os_family: str
    arch: str
    logical_cpu_cores: int
    total_memory_bytes: int
    available_memory_bytes: int
    detected_backends: tuple[AcceleratorBackend, ...]

    @property
    def safe_memory_limit_bytes(self) -> int:
        return (self.available_memory_bytes // 100) * 85

    def can_admit_model(self, required_bytes: int) -> bool:
        return 0 < required_bytes <= self.safe_memory_limit_bytes

    def select_execution_backend(
        self,
        preferred: AcceleratorBackend | None = None,
        force_cpu: bool = False,
    ) -> AcceleratorBackend:
        if force_cpu:
            return AcceleratorBackend.CPU
        if preferred is not None and preferred in self.detected_backends:
            return preferred
        for backend in self.detected_backends:
            if not backend.is_cpu:
                return backend
        return AcceleratorBackend.CPU

    def to_dict(self) -> dict[str, Any]:
        return {
            "os_family": self.os_family,
            "arch": self.arch,
            "logical_cpu_cores": self.logical_cpu_cores,
            "total_memory_bytes": self.total_memory_bytes,
            "available_memory_bytes": self.available_memory_bytes,
            "detected_backends": [b.value for b in self.detected_backends],
        }

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> HardwareProfile:
        return cls(
            os_family=str(payload["os_family"]),
            arch=str(payload["arch"]),
            logical_cpu_cores=int(payload["logical_cpu_cores"]),
            total_memory_bytes=int(payload["total_memory_bytes"]),
            available_memory_bytes=int(payload["available_memory_bytes"]),
            detected_backends=tuple(
                AcceleratorBackend(b) for b in payload["detected_backends"]
            ),
        )


@dataclass(frozen=True)
class CloudConsentToken:
    token_id: str
    modality: Modality
    target_endpoint: str
    max_payload_bytes: int
    issued_at_epoch_secs: int
    expires_at_epoch_secs: int
    user_acknowledged: bool

    def is_valid_for(
        self,
        requested_modality: Modality,
        endpoint: str,
        payload_bytes: int,
        current_epoch_secs: int,
    ) -> bool:
        return (
            self.user_acknowledged
            and bool(self.token_id.strip())
            and self.target_endpoint.startswith("https://")
            and self.modality == requested_modality
            and self.target_endpoint == endpoint
            and 0 <= payload_bytes <= self.max_payload_bytes
            and self.issued_at_epoch_secs < self.expires_at_epoch_secs
            and current_epoch_secs < self.expires_at_epoch_secs
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "token_id": self.token_id,
            "modality": self.modality.value,
            "target_endpoint": self.target_endpoint,
            "max_payload_bytes": self.max_payload_bytes,
            "issued_at_epoch_secs": self.issued_at_epoch_secs,
            "expires_at_epoch_secs": self.expires_at_epoch_secs,
            "user_acknowledged": self.user_acknowledged,
        }

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> CloudConsentToken:
        return cls(
            token_id=str(payload["token_id"]),
            modality=Modality(payload["modality"]),
            target_endpoint=str(payload["target_endpoint"]),
            max_payload_bytes=int(payload["max_payload_bytes"]),
            issued_at_epoch_secs=int(payload["issued_at_epoch_secs"]),
            expires_at_epoch_secs=int(payload["expires_at_epoch_secs"]),
            user_acknowledged=bool(payload["user_acknowledged"]),
        )


@dataclass(frozen=True)
class PrivacyPolicyState:
    local_only_mode: bool = True
    force_cpu_only: bool = False
    active_consent: CloudConsentToken | None = None

    def authorize_cloud_execution(
        self,
        modality: Modality,
        endpoint: str,
        payload_bytes: int,
        current_epoch_secs: int,
    ) -> bool:
        if self.local_only_mode or self.active_consent is None:
            return False
        return self.active_consent.is_valid_for(
            requested_modality=modality,
            endpoint=endpoint,
            payload_bytes=payload_bytes,
            current_epoch_secs=current_epoch_secs,
        )

