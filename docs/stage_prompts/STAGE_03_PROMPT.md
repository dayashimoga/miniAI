# Stage 3 Technical Execution Prompt: Offline LLM Inference, Chat, Streaming, Model Switching & Benchmarking

## 1. Objective & Dependencies
- **Objective:** Deliver complete offline LLM inference (`Modality::Llm`) with CPU-first execution, optional GPU acceleration, token-by-token streaming, multi-turn conversation persistence, hot model switching, and quantitative LLM benchmarking.
- **Dependencies:** Stage 2 Gate Verified.
- **Active Skills:** `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-evaluation-benchmarks`, `miniai-flutter-ui`, `miniai-offline-first`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 3.1: Local LLM Runtime Adapter (`crates/miniai-llm`)**
  - *Subphase 3.1.1:* CPU-first LLM inference engine (GGUF / Candle / ONNX) with configurable context window, temperature, top-p, repeat penalty, deterministic seed, and stop sequences.
  - *Subphase 3.1.2:* Hot model switching with verified memory release before loading the replacement model.
- **Phase 3.2: Multi-Session Chat Persistence & Streaming UI**
  - *Subphase 3.2.1:* SQLite conversation history with message editing, branch regeneration, and JSON/Markdown export.
  - *Subphase 3.2.2:* Flutter Chat Workspace with real-time token streaming, cancellation button, and live tokens/sec badge.
- **Phase 3.3: Quantitative LLM Benchmark Suite**
  - *Subphase 3.3.1:* Cold load time, Time-To-First-Token (TTFT), generation throughput (tokens/s), and peak memory RSS tracked and persisted.

## 3. Security & Edge Cases
- Enforce strict prompt/context token limits to prevent OOM crashes.
- Ensure cancellation immediately halts token generation and frees worker threads within `< 100 ms`.

## 4. Acceptance Tests & Exit Criteria
- Offline CPU chat completion and streaming tests pass 100%.
- Benchmark runner produces verified `PROVEN_ON_HOST` metrics JSON.
- >= 90% test coverage on `miniai-llm` and chat controller logic.

