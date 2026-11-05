---
name: miniai-independent-audit
description: >-
  Guides independent quality, security, and architectural auditing using specialist subagents
  (independent-qa-auditor and arch-security-reviewer) and automated verification scripts.
  Activate before signing off on any subphase or stage gate.
---

# MiniAI Independent Auditing Skill

Use this skill to perform an objective, adversarial verification of any completed stage before claiming completion.

## Audit Protocol
1. **Automated Verification Script**:
   - Run `uv run python scripts/verify_stage0.py` to check file existence, append-only log rules, Antigravity skill/rule frontmatter, secret hygiene, and test suites.
2. **Specialist Subagent Invocations**:
   - Invoke `independent-qa-auditor` via `invoke_subagent` to independently run tests, inspect coverage, and verify zero mock/placeholder code.
   - Invoke `arch-security-reviewer` via `invoke_subagent` to audit architectural modularity, consent gates, and CI/CD security.
3. **Zero Tolerance for Unverified Claims**:
   - If the auditor identifies any failing test, coverage below 90%, or missing artifact, remediate the issue and re-run the audit before passing the gate.

## Verification
- Both automated verification (`verify_stage0.py`) and independent subagent audits must return `PASS`.

