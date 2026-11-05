---
name: miniai-github-actions
description: >-
  Guides GitHub Actions CI/CD workflow authoring for MiniAI across Windows, Linux, macOS,
  Android, and iOS targets under .github/workflows/. Activate when configuring build, test,
  security scanning, packaging, or release automation workflows.
---

# MiniAI GitHub Actions Multi-Platform CI/CD Skill

Use this skill when creating or modifying `.github/workflows/*.yml`.

## CI/CD Engineering Rules
1. **Appropriate OS Runners**:
   - Windows Desktop & Rust/Flutter/Python: `windows-latest`
   - Linux Desktop & Android SDK builds: `ubuntu-latest`
   - macOS Desktop & iOS Xcode builds (`--no-codesign`): `macos-latest`
2. **Least-Privilege Permissions**:
   - Declare `permissions: contents: read` at the workflow root; elevate only inside release jobs that upload release assets.
3. **Never Expose Secrets**:
   - Never echo environment secrets or tokens in workflow steps.

## Workflow Steps
1. Consult [`references/runner_matrix.md`](./references/runner_matrix.md).
2. Edit or add workflow YAML files in `.github/workflows/`.
3. Run `scripts/verify_stage0.py` to validate workflow structure, runner targets, and permissions blocks.

## Verification
- Confirm `ci.yml`, `security.yml`, and `release.yml` cover Windows, Linux, macOS, Android, and iOS targets and pass local YAML validation.

