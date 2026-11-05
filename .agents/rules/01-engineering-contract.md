# Rule 01: Core Engineering Contract & Append-Only Governance

## Scope
Applies to all directories, source files, scripts, and documentation in `MiniAI`.

## Mandates
1. **Think Before Coding**:
   - Inspect existing files and state assumptions explicitly before writing code.
   - Surface architectural trade-offs and record significant decisions in `ADR.md`.
2. **Simplicity First**:
   - Implement the minimum complete, production-ready solution required for the current stage.
   - Reject speculative abstractions, unused configuration hooks, and dead code.
3. **Surgical Changes**:
   - Touch only the files required by the current task.
   - Preserve existing working tests, comments, and documentation.
4. **Zero Mock / Zero Placeholder Policy**:
   - Never introduce fake/mock data into production paths, hardcoded demo screens, unconnected UI buttons, or simulated success outcomes.
5. **Strictly Append-Only `TODO.md` and `CHANGELOG.md`**:
   - Never overwrite, delete, rewrite, reorder, or edit existing lines in `TODO.md` or `CHANGELOG.md`.
   - Record every task addition, state transition, completion, and release as a new dated entry appended to the end of the file.

