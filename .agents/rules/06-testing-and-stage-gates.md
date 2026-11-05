# Rule 06: Testing, Coverage & Stage Gate Protocol

## Scope
Applies to all stage implementations and verification audits from Stage 0 through Stage 10.

## Mandates
1. **Completion Definition**:
   - A task, subphase, or stage is NEVER complete merely because code was written or CI YAML was generated.
   - Completion requires: **implemented, integrated, executed, tested, documented, and independently verified**.
2. **Coverage & Pass Rate**:
   - Critical business logic (`crates/miniai-core`, `ml/src/miniai_ml`) must achieve >= 90% measured test coverage.
   - Required release and gate tests must achieve a 100% pass rate.
3. **Truthful Verification Classification**:
   - Every test and capability report must classify results accurately as:
     - `PROVEN_ON_HOST`
     - `VERIFIED_IN_CI`
     - `HARDWARE_DEPENDENT`
   - Never fabricate test outputs, coverage metrics, or hardware benchmark numbers.
4. **Gate Audit Sign-Off**:
   - Run `scripts/verify_stage0.py` (and stage-specific verifiers) plus independent subagent audit (`independent-qa-auditor`) before closing any stage gate.

