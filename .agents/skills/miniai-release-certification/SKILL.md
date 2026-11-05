---
name: miniai-release-certification
description: >-
  Guides cross-platform release certification, artifact checksum generation, SBOM inventory,
  and release gate verification for MiniAI (Stages 0 and 10). Activate when preparing release
  candidates, verifying RELEASE_CERTIFICATION.md criteria, or auditing release artifacts.
---

# MiniAI Release Certification Skill

Use this skill when certifying a stage gate or preparing cross-platform release artifacts.

## Certification Requirements
1. **100% Passing Required Tests**: Zero failing unit, contract, integration, UI, or security tests.
2. **Coverage Verification**: Measured >= 90% coverage on critical business logic.
3. **Artifact Checksums**: Every release binary or bundle must have a corresponding `.sha256` digest file.
4. **Honest Evidence Matrix**: Explicitly classify what was proven on host (`PROVEN_ON_HOST`), verified in CI (`VERIFIED_IN_CI`), or hardware-dependent (`HARDWARE_DEPENDENT`).

## Workflow Steps
1. Consult [`references/certification_checklist.md`](./references/certification_checklist.md).
2. Run `uv run python scripts/verify_stage0.py` to generate deterministic verification evidence.
3. Update `RELEASE_CERTIFICATION.md` and append release/gate records to `CHANGELOG.md` and `TODO.md`.

## Verification
- All items in `RELEASE_CERTIFICATION.md` for the current stage must be verified with concrete command outputs.

