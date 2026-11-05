"""Repository governance, Antigravity customization, and append-only log verifier."""

from __future__ import annotations

from pathlib import Path
import re
from typing import Any

REQUIRED_GOVERNANCE_FILES: tuple[str, ...] = (
    "AGENTS.md",
    "GLOBAL_ENGINEERING_CONTRACT.md",
    "CLAUDE.md",
    "REQUIREMENTS.md",
    "ARCHITECTURE.md",
    "IMPLEMENTATION_PLAN.md",
    "CODEBASE_GUIDE.md",
    "USER_GUIDE.md",
    "DEVELOPMENT_SETUP.md",
    "TESTING.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "ADR.md",
    "ROADMAP.md",
    "RELEASE_CERTIFICATION.md",
    "TODO.md",
    "CHANGELOG.md",
    "README.md",
)

REQUIRED_RULES: tuple[str, ...] = (
    ".agents/rules/01-engineering-contract.md",
    ".agents/rules/02-rust-standards.md",
    ".agents/rules/03-flutter-standards.md",
    ".agents/rules/04-python-ml-standards.md",
    ".agents/rules/05-security-and-privacy.md",
    ".agents/rules/06-testing-and-stage-gates.md",
)

REQUIRED_SKILLS: tuple[str, ...] = (
    "miniai-architecture",
    "miniai-rust-core",
    "miniai-flutter-ui",
    "miniai-python-ml",
    "miniai-model-runtimes",
    "miniai-training-distillation",
    "miniai-multimodal",
    "miniai-evaluation-benchmarks",
    "miniai-hardware-optimization",
    "miniai-offline-first",
    "miniai-security-privacy",
    "miniai-containers-dev",
    "miniai-github-actions",
    "miniai-testing-qa",
    "miniai-release-certification",
    "miniai-documentation-logs",
    "miniai-staged-implementation",
    "miniai-independent-audit",
)

REQUIRED_STAGE_PROMPTS: tuple[str, ...] = tuple(
    f"docs/stage_prompts/STAGE_{i:02d}_PROMPT.md" for i in range(11)
)

_ENTRY_TIMESTAMP_RE = re.compile(
    r"^#{2,3}\s+\[(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2})\]",
    re.MULTILINE,
)

_MARKDOWN_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")

_SECRET_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("GitHub Classic PAT", re.compile(r"ghp_[A-Za-z0-9]{36}")),
    ("GitHub Fine-Grained PAT", re.compile(r"github_pat_[A-Za-z0-9_]{50,}")),
    ("AWS Access Key ID", re.compile(r"AKIA[0-9A-Z]{16}")),
    ("Private Key Block", re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----")),
)


def verify_append_only_log(file_path: Path) -> list[str]:
    """Verifies that `TODO.md` or `CHANGELOG.md` conforms to append-only chronological rules."""
    errors: list[str] = []
    if not file_path.is_file():
        return [f"Missing append-only log file: {file_path}"]

    content = file_path.read_text(encoding="utf-8")
    if "APPEND-ONLY GOVERNANCE NOTICE" not in content:
        errors.append(f"{file_path.name}: Missing APPEND-ONLY GOVERNANCE NOTICE header.")

    timestamps = _ENTRY_TIMESTAMP_RE.findall(content)
    if not timestamps:
        errors.append(f"{file_path.name}: No dated entry headers found.")
        return errors

    for idx in range(1, len(timestamps)):
        if timestamps[idx] < timestamps[idx - 1]:
            errors.append(
                f"{file_path.name}: Out-of-order timestamp {timestamps[idx]} after {timestamps[idx - 1]}"
            )
    return errors


def validate_skill_file(skill_dir: Path, expected_name: str) -> list[str]:
    """Validates an Antigravity `SKILL.md` frontmatter and linked reference files."""
    errors: list[str] = []
    skill_md = skill_dir / "SKILL.md"
    if not skill_md.is_file():
        return [f"Missing SKILL.md for skill '{expected_name}' at {skill_md}"]

    text = skill_md.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        errors.append(f"{skill_md}: Missing YAML frontmatter opening '---'")
        return errors

    end_idx = text.find("\n---\n", 4)
    if end_idx == -1:
        errors.append(f"{skill_md}: Missing YAML frontmatter closing '---'")
        return errors

    frontmatter = text[4:end_idx]
    if f"name: {expected_name}" not in frontmatter:
        errors.append(f"{skill_md}: Frontmatter name does not match '{expected_name}'")
    if "description:" not in frontmatter:
        errors.append(f"{skill_md}: Frontmatter missing 'description:' field")

    body = text[end_idx + 5 :]
    for match in _MARKDOWN_LINK_RE.finditer(body):
        target = match.group(1).strip()
        if target.startswith(("http://", "https://", "#")):
            continue
        resolved = (skill_dir / target).resolve()
        if not resolved.exists():
            errors.append(f"{skill_md}: Broken relative link '{target}' -> {resolved}")

    return errors


def scan_for_secret_leaks(root: Path) -> list[str]:
    """Scans repository files for accidental secret or credential patterns."""
    findings: list[str] = []
    skip_dirs = {".git", "target", ".dart_tool", "build", ".venv", "__pycache__", ".pytest_cache"}
    for path in root.rglob("*"):
        if any(part in skip_dirs for part in path.parts) or not path.is_file():
            continue
        if path.suffix.lower() in {".png", ".jpg", ".ico", ".exe", ".dll", ".rlib", ".rmeta", ".bin"}:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        for label, pattern in _SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(f"{label} pattern detected in {path.relative_to(root)}")
    return findings


def audit_repository_governance(root: Path) -> dict[str, Any]:
    """Performs a full structural and governance audit of the MiniAI workspace."""
    errors: list[str] = []

    for rel_file in REQUIRED_GOVERNANCE_FILES:
        fpath = root / rel_file
        if not fpath.is_file() or fpath.stat().st_size == 0:
            errors.append(f"Missing or empty governance document: {rel_file}")

    for rule_rel in REQUIRED_RULES:
        rpath = root / rule_rel
        if not rpath.is_file() or rpath.stat().st_size == 0:
            errors.append(f"Missing or empty Antigravity rule: {rule_rel}")
        elif rpath.stat().st_size > 24_000:
            errors.append(f"Rule exceeds 24 KB Antigravity limit: {rule_rel}")

    skills_index = root / ".agents/skills/SKILLS_INDEX.md"
    if not skills_index.is_file():
        errors.append("Missing .agents/skills/SKILLS_INDEX.md")

    for skill_name in REQUIRED_SKILLS:
        sdir = root / ".agents" / "skills" / skill_name
        errors.extend(validate_skill_file(sdir, skill_name))

    for prompt_rel in REQUIRED_STAGE_PROMPTS:
        ppath = root / prompt_rel
        if not ppath.is_file() or ppath.stat().st_size == 0:
            errors.append(f"Missing or empty stage prompt: {prompt_rel}")

    errors.extend(verify_append_only_log(root / "TODO.md"))
    errors.extend(verify_append_only_log(root / "CHANGELOG.md"))
    errors.extend(scan_for_secret_leaks(root))

    return {
        "passed": len(errors) == 0,
        "governance_files_checked": len(REQUIRED_GOVERNANCE_FILES),
        "rules_checked": len(REQUIRED_RULES),
        "skills_checked": len(REQUIRED_SKILLS),
        "stage_prompts_checked": len(REQUIRED_STAGE_PROMPTS),
        "errors": errors,
    }

