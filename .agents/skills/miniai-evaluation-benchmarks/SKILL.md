---
name: miniai-evaluation-benchmarks
description: >-
  Guides quantitative model evaluation, latency/throughput benchmarking, and hardware-honest
  reporting across all MiniAI modalities (Stages 3, 4, 6, 7, 8, and 10). Activate when designing
  benchmark harnesses, computing quality metrics, or verifying performance budgets.
---

# MiniAI Evaluation & Benchmarking Skill

Use this skill when implementing or running model evaluation suites and performance benchmarks.

## Benchmarking Integrity Rules
1. **Never Fabricate Metrics**: All latency (TTFT, tokens/sec, ms/frame), memory (peak RSS/VRAM), and quality metrics (perplexity, WER, FID, CLIP score) must be measured from real execution.
2. **Explicit Provenance Classification**:
   - Tag every benchmark record with `verification_tier`: `PROVEN_ON_HOST`, `VERIFIED_IN_CI`, or `HARDWARE_DEPENDENT`.
3. **Warmup & Statistical Rigor**: Separate cold-start model load time from warm inference iterations and report p50, p95, and mean values.

## Workflow Steps
1. Read [`references/benchmark_schema.md`](./references/benchmark_schema.md).
2. Capture host hardware profile (`HardwareProfile`) prior to running benchmark iterations.
3. Run warmup pass followed by measured passes.
4. Persist structured benchmark report JSON and verify no regression against baseline thresholds.

## Verification
- Validate benchmark report schema and statistical calculations via `cargo test` and `uv run pytest`.

