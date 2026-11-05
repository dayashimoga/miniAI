---
name: miniai-staged-implementation
description: >-
  Guides the hierarchical Stage -> Phase -> Subphase -> Tasks -> Acceptance Tests execution loop
  for MiniAI. Activate at the beginning and end of every stage, phase, or subphase to ensure
  complete, non-fragmented delivery and gate verification.
---

# MiniAI Staged Implementation Execution Skill

Use this skill to execute any Stage, Phase, or Subphase in `IMPLEMENTATION_PLAN.md` and `docs/stage_prompts/`.

## The 7-Step Subphase Execution Loop
1. **Define Success Criteria**: Read the stage prompt in `docs/stage_prompts/STAGE_XX_PROMPT.md` and list measurable acceptance tests.
2. **Implement**: Build the complete vertical slice without mocks or placeholders.
3. **Integrate**: Connect Rust core, Flutter UI, and Python ML contracts as required by the stage.
4. **Run Tests**: Execute unit, contract, integration, UI, offline, and security tests.
5. **Fix Failures**: Resolve all compiler warnings, lint errors, test failures, or coverage gaps.
6. **Audit Regressions & Security**: Invoke `independent-qa-auditor` and `arch-security-reviewer`.
7. **Update Docs & Append-Only Logs**: Append completion evidence to `TODO.md` and `CHANGELOG.md` and emit the Stage Gate Report.

## Verification
- Refer to [`references/stage_gate_report_template.md`](./references/stage_gate_report_template.md) for the mandatory gate report structure.

