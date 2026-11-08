---
name: miniai-ux-accessibility
description: >-
  Guides accessible UI design, WCAG 2.1 AA compliance, semantics labeling, and ergonomic interactions
  across all MiniAI Flutter screens under ui/miniai_app/.
---

# MiniAI UX & Accessibility Engineering Skill

Use this skill when designing widgets, implementing semantics, auditing color contrast, or ensuring WCAG 2.1 AA compliance.

## Accessibility Guidelines
1. **Semantics & Screen Readers**:
   - Wrap interactive chips, icon buttons, and custom controls with `Semantics(label: ..., button: true)`.
   - Ensure dynamic state changes (e.g. Privacy Mode toggle, model loading status, download progress) announce informative status updates to assistive technologies.
2. **Touch Targets & Contrast**:
   - Maintain minimum tap targets of $48\times 48\text{dp}$ on mobile and $32\times 32\text{dp}$ on desktop.
   - Contrast ratio must meet or exceed 4.5:1 for normal text and 3:1 for large text across light and dark Material 3 color schemes.
3. **Reasoning & Content Presentation**:
   - Format AI reasoning steps inside collapsible `<think>` accordions so users are not overwhelmed by verbose traces.
   - Display grounded citations as clickable chips with source document names and line number anchors.
4. **Keyboard Navigability**:
   - All interactive elements must be reachable via `Tab` key navigation and actionable via `Space` / `Enter`.

## Verification
- Run `cd ui/miniai_app; flutter test test/widget_test.dart` to verify accessibility semantics and labels.
- Run `cd ui/miniai_app; flutter test test/chat_workspace_test.dart` to verify citation chips and message interactions.

