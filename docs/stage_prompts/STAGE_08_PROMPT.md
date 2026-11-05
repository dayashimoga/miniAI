# Stage 8 Technical Execution Prompt: Video Understanding, Generation & Media Workflows

## 1. Objective & Dependencies
- **Objective:** Deliver independent Video Understanding & Generation (`Modality::Video`) runtime adapter, memory-bounded frame sampling, and composable multimodal media workflows.
- **Dependencies:** Stage 7 Gate Verified.
- **Active Skills:** `miniai-multimodal`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-evaluation-benchmarks`, `miniai-flutter-ui`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 8.1: Video Frame Sampling & Temporal Understanding (`crates/miniai-video`)**
  - *Subphase 8.1.1:* Memory-bounded keyframe extractor, audio track demuxer, and temporal scene description/QA pipeline.
- **Phase 8.2: Video Synthesis & Multimodal DAG Workflows**
  - *Subphase 8.2.1:* Local prompt/image-to-video adapter and composable media pipeline orchestrator chaining LLM, Image, TTS, Music, and Video adapters.
- **Phase 8.3: Flutter Video Timeline & Workflow Studio**
  - *Subphase 8.3.1:* Keyframe scrubber, temporal event markers, and DAG workflow execution monitor.

## 3. Security & Edge Cases
- Enforce strict frame buffer pools so high-resolution video files cannot trigger host OOM.
- Clean up temporary frame extraction directories atomically on job completion or cancellation.

## 4. Acceptance Tests & Exit Criteria
- Video frame extraction and temporal analysis stay within configured memory budget on CPU.
- Multi-modality DAG workflow executes end-to-end offline.
- >= 90% coverage on `miniai-video` and workflow orchestration logic.

