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
from .dataset import (
    DatasetManifest,
    DatasetRecord,
    compute_file_sha256,
    deduplicate_records,
    load_dataset_file,
    save_splits_and_manifest,
    split_dataset,
    validate_record,
    verify_dataset_manifest,
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
    "DatasetManifest",
    "DatasetRecord",
    "HardwareProfile",
    "Modality",
    "PrivacyPolicyState",
    "VerificationTier",
    "REQUIRED_GOVERNANCE_FILES",
    "REQUIRED_RULES",
    "REQUIRED_SKILLS",
    "REQUIRED_STAGE_PROMPTS",
    "audit_repository_governance",
    "compute_file_sha256",
    "compute_sha256_hex",
    "deduplicate_records",
    "load_dataset_file",
    "probe_ml_environment",
    "save_splits_and_manifest",
    "scan_for_secret_leaks",
    "seed_everything",
    "split_dataset",
    "validate_record",
    "validate_safe_relative_path",
    "verify_append_only_log",
    "verify_dataset_manifest",
    "verify_sha256_digest",
]

