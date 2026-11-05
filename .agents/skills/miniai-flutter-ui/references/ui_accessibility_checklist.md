# Flutter UI Accessibility & Quality Checklist

1. **Semantic Labels**:
   - Privacy mode toggle must expose current status (`Local-Only Privacy Mode: Active` or `Inactive`).
   - Modality cards must expose modality name, active backend (`CPU`, `CUDA`, `Vulkan`, `Metal`), and health status.
2. **Adaptive Breakpoints**:
   - `< 600px`: Compact single-column layout with bottom/drawer navigation.
   - `600px - 1024px`: Tablet two-column adaptive layout.
   - `> 1024px`: Desktop multi-pane shell with persistent navigation rail and status bar.
3. **Error Recovery**:
   - Display actionable remediation guidance when an operation is blocked by `LocalOnlyPrivacyMode` or missing model weights.

