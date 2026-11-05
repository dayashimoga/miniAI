---
name: miniai-hardware-optimization
description: >-
  Guides hardware capability detection, CPU-first fallback logic, SIMD/thread scheduling,
  and optional GPU/NPU acceleration in MiniAI. Activate during Stages 2, 3, 6, 7, 8, and 9
  when probing hardware or selecting execution backends.
---

# MiniAI Hardware Probing & Optimization Skill

Use this skill when implementing hardware detection, memory budgeting, or backend selection logic.

## Core Hardware Principles
1. **Never Assume a GPU**: Every code path must succeed when only `AcceleratorBackend::Cpu` is present.
2. **Dynamic Probing & Graceful Fallback**:
   - Probe available CPU cores/threads, system RAM, and optional accelerators (CUDA, Vulkan, Metal, DirectML, ROCm, NPU).
   - If a requested hardware backend fails initialization or runs out of VRAM, automatically fall back to `AcceleratorBackend::Cpu` and emit a structured diagnostic event.
3. **Memory Safety Guard**: Estimate model weight footprint before loading and warn/block if required memory exceeds safe available RAM/VRAM thresholds.

## Workflow Steps
1. Consult [`references/backend_selection_policy.md`](./references/backend_selection_policy.md).
2. Query `HardwareDetector` for the host `HardwareProfile`.
3. Select the optimal backend via `select_execution_backend` with deterministic fallback to `Cpu`.
4. Test both accelerated selection and forced-CPU fallback in unit tests.

## Verification
- Execute `cargo test --workspace` to verify backend selection and memory guard edge cases.

