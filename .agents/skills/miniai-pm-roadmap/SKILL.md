---
name: miniai-pm-roadmap
description: >-
  Guides Product Management, milestone scheduling, gap prioritization, and roadmap synchronization
  for MiniAI across Stages 0 through 15 and v2.0 roadmap releases.
---

# MiniAI Product Management & Roadmap Skill

Use this skill when tracking product delivery milestones, prioritizing technical debt, auditing production gaps, or maintaining roadmap documents (`PRODUCTION_AUDIT_AND_V2_ROADMAP.md`, `ROADMAP.md`, `TODO.md`, and `CHANGELOG.md`).

## Core Responsibilities
1. **Milestone Integrity**:
   - Ensure every stage (0 through 15) adheres strictly to the Stage -> Phase -> Subphase -> Tasks -> Acceptance Criteria progression.
   - Maintain the strictly append-only logging requirement for `TODO.md` and `CHANGELOG.md`.
2. **Gap Analysis & Prioritization**:
   - Triage production audit findings across the 5 core remediation phases:
     - Phase A: Real neural runtime execution with genuine weights and zero fake telemetry.
     - Phase B: Intent routing and conversational RAG gating to prevent spurious citations.
     - Phase C: Verified registry downloads with TLS redirect handling and verified SHA-256 digests.
     - Phase D: UX enhancements (collapsible reasoning traces, desktop shortcuts, memory pressure handling).
     - Phase E: Cross-platform release packaging, SBOM generation, and CI/CD artifact distribution.
3. **Release Certification**:
   - Ensure all quality gates (100% test pass rate, $\ge 90\%$ critical logic coverage, zero unhandled errors) are met before tag creation.

## Workflow
1. Read the latest requirements from [`PRODUCTION_AUDIT_AND_V2_ROADMAP.md`](../../../PRODUCTION_AUDIT_AND_V2_ROADMAP.md).
2. Verify task status in [`TODO.md`](../../../TODO.md) and ensure no existing entries are edited or removed.
3. Coordinate verification runs across Rust core (`cargo test`), Flutter UI (`flutter test`), and Python ML (`pytest`).
4. Append dated release logs to [`CHANGELOG.md`](../../../CHANGELOG.md) upon milestone sign-off.

