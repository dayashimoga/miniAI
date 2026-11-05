# Hardware Backend Selection Policy

1. **Explicit User Override**: If the user explicitly selects `CPU Only`, always return `AcceleratorBackend::Cpu` regardless of detected GPUs.
2. **Preferred Accelerator Order (when auto-selection is enabled)**:
   - macOS / iOS: `Metal` -> `CoreMl` -> `Cpu`
   - Windows: `Cuda` -> `DirectMl` -> `Vulkan` -> `Cpu`
   - Linux: `Cuda` -> `Rocm` -> `Vulkan` -> `Cpu`
   - Android: `Vulkan` -> `Nnapi` -> `Cpu`
3. **RAM/VRAM Admission Control**:
   - Require `estimated_model_bytes <= available_memory_bytes * 0.85` for safe admission.

