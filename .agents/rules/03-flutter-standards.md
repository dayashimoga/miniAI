# Rule 03: Flutter Cross-Platform UI Standards

## Scope
Applies to the Flutter application under `ui/miniai_app/`.

## Mandates
1. **Cross-Platform Targets**:
   - Support Windows, Linux, macOS, Android, and iOS from a clean, responsive layout system.
2. **No Placeholder Controls**:
   - Every visible widget, button, toggle, or status indicator must connect to a real state controller and core contract.
   - Do not render dummy chat messages, fake download progress bars, or hardcoded benchmark charts.
3. **Accessibility & Responsiveness**:
   - Provide `Semantics` labels for interactive controls, status indicators, and privacy mode badges.
   - Support keyboard navigation, high-contrast themes, and adaptive breakpoints (mobile, tablet, desktop).
4. **Verification**:
   - Every commit touching Flutter code must pass `flutter analyze --no-pub` with zero issues and `flutter test` with 100% passing widget and unit tests.

