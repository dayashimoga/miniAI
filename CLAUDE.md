# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make them pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## 5. MiniAI Project-Specific Integration Rules

This file is integrated with `GLOBAL_ENGINEERING_CONTRACT.md`, `AGENTS.md`, and adapted to Antigravity via `GEMINI.md`. Every contributor and AI agent working on **MiniAI** must additionally adhere to:

0. **Antigravity Pairing & Customization Integration (`GEMINI.md`)**:
   - In Antigravity environments, behavioral guidelines are enforced via [`GEMINI.md`](./GEMINI.md), Antigravity rules (`.agents/rules/*.md`), and reusable skills (`.agents/skills/*/SKILL.md`).
   - Use clickable `file:///` file links, create structured markdown artifacts in `<appDataDir>\brain\<conversation-id>`, and delegate independent audits to subagents (`independent-qa-auditor`, `arch-security-reviewer`).
1. **Append-Only Logs (`TODO.md` and `CHANGELOG.md`)**:
   - Never overwrite, delete, rewrite, reorder, or silently modify existing entries in `TODO.md` or `CHANGELOG.md`.
   - Record every addition, completion, status transition, and release as a new dated entry at the bottom of the file.
2. **Real Implementations Only (Zero Mock/Placeholder Policy)**:
   - Never commit fake/mock data in production paths, hardcoded demonstrations, placeholder screens/buttons, simulated success responses, or unconnected UI controls.
3. **Architecture & Runtime Independence**:
   - Rust (`crates/`) powers core services, storage, hardware detection, model registry, and runtime orchestration.
   - Flutter (`ui/miniai_app/`) powers the cross-platform user interface across Windows, Linux, macOS, Android, and iOS.
   - Python/PyTorch (`ml/`) powers training, fine-tuning, distillation, dataset processing, and evaluation tooling.
   - LLM, vision, image generation, audio/speech, music, and video models use independently pluggable runtime adapters (`ModalityAdapter` trait) and are never forced to share a monolithic runtime.
4. **Local-First, CPU-First, Consent-Gated Cloud**:
   - Default execution is 100% local and CPU-compatible.
   - Hardware accelerators (CUDA, Metal, Vulkan, ROCm, DirectML, NPU) are optional and dynamically probed.
   - Cloud or distributed compute requires explicit, revocable, per-session/per-task user consent and must never activate when `LocalOnlyPrivacyMode` is enabled.
5. **Stage Gate Verification**:
   - A subphase or stage is complete only when **implemented, integrated, executed, tested, documented, and independently verified** with >= 90% measured coverage on critical business logic and 100% passing required tests.

