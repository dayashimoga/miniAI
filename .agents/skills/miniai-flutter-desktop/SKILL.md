---
name: miniai-flutter-desktop
description: >-
  Guides desktop-specific Flutter engineering for MiniAI across Windows (Win32), Linux (GTK),
  and macOS (AppKit). Activate when implementing multi-pane layouts, desktop shortcuts, window lifecycle,
  or native loopback IPC communication.
---

# MiniAI Flutter Desktop Engineering Skill

Use this skill when building, testing, or optimizing desktop-specific experiences in `ui/miniai_app/`.

## Desktop Invariants
1. **Adaptive Multi-Pane Layout**:
   - For viewports $\ge 760\text{px}$, render the persistent sidebar with session management, model selection, and memory inspector.
   - For viewports $< 760\text{px}$, automatically collapse navigation into an accessible drawer.
2. **Keyboard Accelerators**:
   - Enforce standard desktop accelerators:
     - `Ctrl+N` (or `Cmd+N` on macOS) to create a new conversation session.
     - `Ctrl+K` (or `Cmd+K` on macOS) to trigger quick model switching or search.
     - `Enter` to submit prompt, `Shift+Enter` for multiline input.
3. **OS Resource & Window Lifecycle**:
   - Register `WidgetsBindingObserver` to monitor system lifecycle and memory pressure.
   - Trigger `controller.unloadActiveModel()` on `didHaveMemoryPressure()` to prevent OS out-of-memory termination.
4. **Desktop Native IPC**:
   - Communicate with the native Rust core via loopback HTTP/WebSocket (`http://127.0.0.1:4140`) with fallback to local in-process client during tests.

## Verification
- Run `cd ui/miniai_app; flutter test test/shell_and_settings_test.dart` to verify desktop navigation rail.
- Run `cd ui/miniai_app; flutter test test/chat_workspace_test.dart` to verify shortcuts and layout adaptation.

