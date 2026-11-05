# MiniAI — Antigravity Agent Guidelines (`GEMINI.md`)

This file is automatically loaded by Antigravity as the primary workspace rule file for `g:\miniAI`. It integrates and adapts the principles of `CLAUDE.md`, `GLOBAL_ENGINEERING_CONTRACT.md`, and `AGENTS.md` to Antigravity's agentic pairing environment.

---

## 1. Antigravity Behavioral Tenets (Adapted from `CLAUDE.md`)

### 1.1 Think Before Coding
- **Inspect First**: Use `read_file` and `run_command` to inspect existing code, schemas, and tests before making modifications.
- **State Assumptions & Surface Trade-offs**: Never make silent assumptions regarding architecture, toolchains, or data formats. If ambiguity exists, present options to the user.
- **Architectural Traceability**: Record non-trivial design decisions in `ADR.md`.

### 1.2 Simplicity First
- **Minimum Complete Solution**: Write the concise, robust code that directly satisfies the active task or subphase acceptance criteria.
- **No Speculative Abstractions**: Avoid generic wrappers, unused configuration options, or speculative extension points.
- **Code Budget**: If a solution can be implemented cleanly in 50 lines, do not write 200 lines.

### 1.3 Surgical Changes
- **Targeted Edits**: Use `edit_file` for targeted replacements. Avoid rewriting entire files when changing localized logic.
- **Preserve Existing Quality**: Preserve all existing comments, docstrings, formatting, and working tests.
- **Clean Up Own Scaffolding**: Remove any temporary imports, dead variables, or test fixtures introduced during your changes. Do not delete pre-existing code unless requested.

### 1.4 Goal-Driven Execution
- **Verifiable Subphase Goals**: Transform each task into measurable acceptance criteria:
  ```
  1. [Step] -> verify: [command check]
  2. [Step] -> verify: [command check]
  ```
- **Loop Until Verified**: Run build, lint, and test commands (`cargo test`, `uv run pytest`, `flutter test`), inspect failures, fix root causes, and re-verify until 100% green.

---

## 2. Antigravity Tool & Workflow Integration

### 2.1 File Links & Symbol References
- Always use clickable markdown links with the `file:///` scheme for all paths and code symbols (e.g., [`miniai-core`](file:///g:/miniAI/crates/miniai-core/src/lib.rs#L10-L30)).
- On Windows, use forward slashes (`file:///g:/miniAI/...`).

### 2.2 Artifacts Management
- Use artifacts for extensive multi-phase reports, architectural diagrams (Mermaid), and experiment logs.
- Write artifacts to the designated artifact directory (`<appDataDir>\brain\<conversation-id>`).
- When creating or editing artifacts, provide complete `ArtifactMetadata`. Do not re-summarize entire artifacts in chat; point the user to key decisions.

### 2.3 Specialist Subagents (`independent-qa-auditor` & `arch-security-reviewer`)
- Use `invoke_subagent` to delegate independent QA and security auditing at stage/subphase gates.
- Rely on reactive wakeups when subagents report completion; never poll in busy loops.
- Treat QA auditor findings as binding: no subphase or stage gate advances without an independent `PASS` verdict.

### 2.4 Slash Commands Recommendations
- Proactively suggest relevant slash commands to the user when appropriate (e.g., `/plan` for multi-step breakdowns, `/goal` for autonomous stage completion, `/compact` to manage context).

---

## 3. MiniAI Non-Negotiable Engineering Rules

1. **Strictly Append-Only `TODO.md` and `CHANGELOG.md`**:
   - Never overwrite, delete, rewrite, or modify existing entries in `TODO.md` or `CHANGELOG.md`.
   - Every task state transition (`PLANNED` -> `IN_PROGRESS` -> `COMPLETED` / `VERIFIED`) must be appended as a new dated entry at the end of the file.
2. **Zero Mock / Zero Placeholder Policy**:
   - Every UI widget, button, toggle, and backend service must use real, tested implementations.
   - Never return fake data or simulate success.
3. **Local-First, CPU-First, & Consent-Gated Cloud**:
   - Every modality must execute deterministically on CPU (`AcceleratorBackend::Cpu`).
   - `LocalOnlyPrivacyMode` is enabled by default. Outbound cloud compute is strictly blocked unless explicitly disabled and authorized by a valid `CloudConsentToken`.
4. **Independently Pluggable Modality Adapters**:
   - All 7 modalities (`llm`, `vision`, `image_generation`, `speech_asr`, `speech_tts`, `music`, `video`) operate via decoupled `ModalityAdapter` instances.
5. **Coverage & Test Pass Rate**:
   - Maintain `>= 90%` measured coverage on critical business logic.
   - Maintain `100%` passing tests across all test suites.

