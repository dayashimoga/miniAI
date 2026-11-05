"""MiniAI Python ML & Governance Foundation Package."""

from .contracts import (
    AcceleratorBackend,
    CloudConsentToken,
    HardwareProfile,
    Modality,
    PrivacyPolicyState,
    VerificationTier,
    compute_sha256_hex,
    validate_safe_relative_path,
    verify_sha256_digest,
)
from .governance import (
     REQUIRED_GOVERNANCE_FILES,
    REQUIRED_RULES,
    REQUIRED_SKILLS,
    REQUIRED_STAGE_PROMPTS,
    audit_repository_governance,
    scan_for_secret_leaks,
    verify_append_only_log,
)
from .runtime_probe import probe_ml_environment, seed_everything

__all__ = [
    "AcceleratorBackend",
    "CloudConsentToken",
    "HardwareProfile",
    "Modality",
    "PrivacyPolicyState",
    "VerificationTier",
    "REQUIRED_GOVERNANCE_FILES",
    "REQUIRED_RULES",
    "REQUIRED_SKILLS",
    "REQUIRED_STAGE_PROMPTS",
    "audit_repository_governance",
    "compute_sha256_hex",
    "probe_ml_environment",
    "scan_for_secret_leaks",
    "seed_everything",
    "validate_safe_relative_path",
    "verify_append_only_log",
    "verify_sha256_digest",
]

