---
name: miniai-flutter-ui
description: >-
  Guides cross-platform Flutter UI engineering for MiniAI across Windows, Linux, macOS, Android,
  and iOS under ui/miniai_app/. Activate when building screens, state controllers, accessibility
  semantics, or widget tests.
---

# MiniAI Flutter Cross-Platform UI Skill

Use this skill when building or updating Flutter UI components in `ui/miniai_app/`.

## Core UI Mandates
1. **Zero Unconnected Controls**: Every button, switch, or card must bind to a real state model and invoke verified domain logic.
2. **Privacy First Visibility**: Always display the current privacy state (`Local-Only Privacy Mode` vs `Hybrid Consent-Gated Mode`) clearly in the shell header/status bar with accessible semantics.
3. **Responsive & Accessible**:
   - Support desktop split-pane and mobile compact layouts (`LayoutBuilder` / adaptive navigation).
   - Provide `Semantics` annotations and tooltips on interactive widgets.

## Workflow Steps
1. Read [`references/ui_accessibility_checklist.md`](./references/ui_accessibility_checklist.md).
2. Implement models, controllers, and widgets under `ui/miniai_app/lib/`.
3. Write comprehensive widget and unit tests under `ui/miniai_app/test/`.
4. Run static analysis:
   `flutter analyze --no-pub` (inside `ui/miniai_app`)
5. Run widget and unit tests:
   `flutter test` (inside `ui/miniai_app`)

## Verification
- `flutter analyze --no-pub` must report `No issues found!`.
- `flutter test` must report 100% passing tests.

