# MiniAI — Contributor & Engineering Workflow Guide (`CONTRIBUTING.md`)

**Document ID:** `MINIAI-CONTRIB-001`  
**Version:** `1.0.0`

---

## 1. Engineering Contract Compliance

All contributors (human engineers and AI agents) must read and follow:
1. [`CLAUDE.md`](./CLAUDE.md) — Think Before Coding, Simplicity First, Surgical Changes, Goal-Driven Execution.
2. [`GLOBAL_ENGINEERING_CONTRACT.md`](./GLOBAL_ENGINEERING_CONTRACT.md) — Zero mock/placeholder policy, CPU-first execution, >=90% coverage on critical logic, 100% passing tests.
3. [`AGENTS.md`](./AGENTS.md) — Antigravity workspace rules (`.agents/rules/`) and reusable skills (`.agents/skills/`).

---

## 2. Branching & Pull Request Workflow

- **`main`**: Protected production-ready branch containing only stage-gate-verified commits.
- **`develop`**: Active integration branch for staged engineering work.
- **Feature/Stage Branches**: Named `stage-<N>/<phase>-<short-description>`.

---

## 3. Strictly Append-Only Logs (`TODO.md` & `CHANGELOG.md`)

**CRITICAL RULE:**
- Never overwrite, delete, rewrite, reorder, or edit existing lines in `TODO.md` or `CHANGELOG.md`.
- Always append a new dated block at the end of `TODO.md` and `CHANGELOG.md` to record newly added tasks, status transitions (`PLANNED` -> `IN_PROGRESS` -> `VERIFIED`), or release milestones.
- Run `uv run --project ml python scripts/verify_stage0.py` before committing to ensure append-only log integrity passes.

---

## 4. Pre-Commit Verification Checklist

Before submitting any change or advancing a subphase:
```powershell
# 1. Rust formatting, linting, and tests
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 2. Python ML tests & >=90% coverage check
Push-Location ml
uv run pytest --cov=miniai_ml --cov-report=term-missing --cov-fail-under=90
Pop-Location

# 3. Flutter static analysis & widget tests
Push-Location ui/miniai_app
flutter analyze --no-pub
flutter test --coverage
Pop-Location

# 4. Master stage gate verification
uv run --project ml python scripts/verify_stage0.py
```

