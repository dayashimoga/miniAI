"""Comprehensive test suite for MiniAI Python ML & Governance foundation (`miniai_ml`)."""

from __future__ import annotations

from pathlib import Path
import pytest

from miniai_ml import (
    AcceleratorBackend,
    CloudConsentToken,
    HardwareProfile,
    Modality,
    PrivacyPolicyState,
    VerificationTier,
    audit_repository_governance,
    compute_sha256_hex,
    probe_ml_environment,
    scan_for_secret_leaks,
    seed_everything,
    validate_safe_relative_path,
    verify_append_only_log,
    verify_sha256_digest,
)
from miniai_ml.governance import validate_skill_file


def test_modality_and_backend_enums() -> None:
    assert len(list(Modality)) == 7
    assert Modality.LLM.value == "llm"
    assert AcceleratorBackend.CPU.is_cpu is True
    assert AcceleratorBackend.CUDA.is_cpu is False
    assert VerificationTier.PROVEN_ON_HOST.value == "PROVEN_ON_HOST"


def test_sha256_integrity_and_path_validation() -> None:
    payload = b"miniai-stage0-test-artifact"
    digest = compute_sha256_hex(payload)
    assert len(digest) == 64
    assert verify_sha256_digest(payload, digest) == digest

    with pytest.raises(ValueError, match="Invalid SHA-256"):
        verify_sha256_digest(payload, "bad-digest")

    with pytest.raises(ValueError, match="integrity mismatch"):
        verify_sha256_digest(b"tampered", digest)

    assert validate_safe_relative_path("models\\llm\\model.gguf") == "models/llm/model.gguf"
    for bad_path in ["", "../escape", "/abs/path", "\\\\unc\\share", "C:\\win\\path", "a/./b"]:
        with pytest.raises(ValueError):
            validate_safe_relative_path(bad_path)


def test_hardware_profile_admission_and_backend_selection() -> None:
    profile = HardwareProfile(
        os_family="windows",
        arch="x86_64",
        logical_cpu_cores=8,
        total_memory_bytes=16_000,
        available_memory_bytes=10_000,
        detected_backends=(AcceleratorBackend.CPU, AcceleratorBackend.VULKAN),
    )
    assert profile.safe_memory_limit_bytes == 8_500
    assert profile.can_admit_model(8_000) is True
    assert profile.can_admit_model(0) is False
    assert profile.can_admit_model(9_000) is False

    assert profile.select_execution_backend(AcceleratorBackend.CUDA, force_cpu=False) == AcceleratorBackend.VULKAN
    assert profile.select_execution_backend(AcceleratorBackend.VULKAN, force_cpu=False) == AcceleratorBackend.VULKAN
    assert profile.select_execution_backend(AcceleratorBackend.VULKAN, force_cpu=True) == AcceleratorBackend.CPU

    cpu_only_profile = HardwareProfile(
        os_family="linux",
        arch="x86_64",
        logical_cpu_cores=4,
        total_memory_bytes=8_000,
        available_memory_bytes=4_000,
        detected_backends=(AcceleratorBackend.CPU,),
    )
    assert cpu_only_profile.select_execution_backend() == AcceleratorBackend.CPU

    round_trip = HardwareProfile.from_dict(profile.to_dict())
    assert round_trip == profile


def test_privacy_policy_and_cloud_consent_token() -> None:
    token = CloudConsentToken(
        token_id="tok-100",
        modality=Modality.LLM,
        target_endpoint="https://compute.miniai.local",
        max_payload_bytes=4096,
        issued_at_epoch_secs=1000,
        expires_at_epoch_secs=2000,
        user_acknowledged=True,
    )
    assert CloudConsentToken.from_dict(token.to_dict()) == token

    # Blocked when local_only_mode is True (even with a token)
    locked_state = PrivacyPolicyState(local_only_mode=True, active_consent=token)
    assert (
        locked_state.authorize_cloud_execution(
            Modality.LLM, "https://compute.miniai.local", 1024, 1500
        )
        is False
    )

    # Allowed when local_only_mode is False and token matches
    hybrid_state = PrivacyPolicyState(local_only_mode=False, active_consent=token)
    assert (
        hybrid_state.authorize_cloud_execution(
            Modality.LLM, "https://compute.miniai.local", 1024, 1500
        )
        is True
    )
    # Expired or wrong modality blocked
    assert (
        hybrid_state.authorize_cloud_execution(
            Modality.VISION, "https://compute.miniai.local", 1024, 1500
        )
        is False
    )
    assert (
        hybrid_state.authorize_cloud_execution(
            Modality.LLM, "https://compute.miniai.local", 1024, 2500
        )
        is False
    )


def test_runtime_probe_and_deterministic_seed() -> None:
    assert seed_everything(1234) == 1234
    with pytest.raises(ValueError):
        seed_everything(-1)

    env_info = probe_ml_environment(force_cpu=True)
    assert env_info["selected_backend"] == "cpu"
    assert env_info["logical_cpu_cores"] >= 1


def test_governance_auditor_and_edge_cases(tmp_path: Path) -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report = audit_repository_governance(repo_root)
    assert report["passed"] is True, f"Governance audit failed: {report['errors']}"

    # Edge cases on temporary files
    missing_log = verify_append_only_log(tmp_path / "MISSING.md")
    assert len(missing_log) == 1

    bad_log = tmp_path / "TODO.md"
    bad_log.write_text(
        "APPEND-ONLY GOVERNANCE NOTICE\n"
        "### [2026-11-05T12:00:00+05:30] Later\n"
        "### [2026-11-05T10:00:00+05:30] Earlier\n",
        encoding="utf-8",
    )
    log_errors = verify_append_only_log(bad_log)
    assert any("Out-of-order timestamp" in e for e in log_errors)

    bad_skill_dir = tmp_path / "bad-skill"
    bad_skill_dir.mkdir()
    assert len(validate_skill_file(bad_skill_dir, "bad-skill")) == 1

    (bad_skill_dir / "SKILL.md").write_text(
        "---\nname: wrong-name\ndescription: test\n---\n[broken](./missing.md)\n[web](https://example.com)\n",
        encoding="utf-8",
    )
    skill_errors = validate_skill_file(bad_skill_dir, "bad-skill")
    assert len(skill_errors) == 2

    # Additional frontmatter edge cases
    no_fm_dir = tmp_path / "no-fm"
    no_fm_dir.mkdir()
    (no_fm_dir / "SKILL.md").write_text("# No frontmatter\n", encoding="utf-8")
    assert len(validate_skill_file(no_fm_dir, "no-fm")) == 1

    unclosed_fm_dir = tmp_path / "unclosed-fm"
    unclosed_fm_dir.mkdir()
    (unclosed_fm_dir / "SKILL.md").write_text("---\nname: unclosed-fm\n", encoding="utf-8")
    assert len(validate_skill_file(unclosed_fm_dir, "unclosed-fm")) == 1

    no_Header_log = tmp_path / "NO_HEADER.md"
    no_Header_log.write_text("plain text without notice or timestamps\n", encoding="utf-8")
    assert len(verify_append_only_log(no_Header_log)) == 2

    # Empty repo audit detects missing files
    empty_audit = audit_repository_governance(tmp_path)
    assert empty_audit["passed"] is False
    assert len(empty_audit["errors"]) > 10

    # Secret scanner detects synthetic token in tmp_path
    fake_token = "ghp_" + ("A" * 36)
    (tmp_path / "leak.txt").write_text(fake_token, encoding="utf-8")
    (tmp_path / "skip.png").write_bytes(b"\x89PNG\r\n")
    leaks = scan_for_secret_leaks(tmp_path)
    assert len(leaks) == 1
