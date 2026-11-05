---
name: miniai-documentation-logs
description: >-
  Enforces the strictly append-only logging rules for TODO.md and CHANGELOG.md and maintains
  synchronization across MiniAI documentation files. Activate whenever recording task progress,
  completing a subphase/stage, or updating architectural and user documentation.
---

# MiniAI Documentation & Append-Only Logs Skill

Use this skill whenever updating `TODO.md`, `CHANGELOG.md`, or living engineering documentation.

## Strictly Append-Only Mandate (`TODO.md` & `CHANGELOG.md`)
1. **NEVER** overwrite, delete, rewrite, reorder, or modify any existing line in `TODO.md` or `CHANGELOG.md`.
2. **NEVER** toggle an existing checkbox `[ ]` to `[x]` in an earlier entry. Instead, append a new dated entry at the very end of the file recording the status transition (`PLANNED` -> `IN_PROGRESS` -> `COMPLETED` / `VERIFIED`).
3. Always use `write_file` with `Append: true` (or append at EOF) when updating `TODO.md` and `CHANGELOG.md`.

## Entry Format Template
Read [`references/append_only_templates.md`](./references/append_only_templates.md) for the exact markdown format required by the automated log parser.

## Verification
- Run `uv run python scripts/verify_stage0.py` to verify chronological append-only integrity of `TODO.md` and `CHANGELOG.md`.

