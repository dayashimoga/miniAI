# MiniAI — Architectural Decision Records (`ADR.md`)

This document records binding architectural decisions for **MiniAI**, including context, considered alternatives, trade-offs, and consequences.

---

## ADR-001: Tri-Language Modular Architecture (Rust Core + Flutter UI + Python/PyTorch ML)
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** MiniAI requires memory-safe, low-overhead local systems orchestration, a responsive native UI across 5 operating systems (Windows, Linux, macOS, Android, iOS), and first-class access to the PyTorch training/distillation ecosystem.
- **Decision:**
  - Use **Rust** (`crates/`) for the core supervisor, model registry, hardware probing, cryptographic SHA-256 verification, privacy enforcement, and runtime orchestration.
  - Use **Flutter** (`ui/miniai_app/`) for the cross-platform desktop and mobile UI.
  - Use **Python/PyTorch** (`ml/`, managed via `uv`) for dataset pipelines, LoRA/QLoRA fine-tuning, distillation, and evaluation harnesses.
- **Consequences:** Strong compile-time safety in core services, single-codebase multi-OS UI, and zero compromise on ML training tooling. Cross-language contracts are enforced via canonical `snake_case` JSON schemas tested across all three languages.

---

## ADR-002: Independently Pluggable Modality Runtime Adapters
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** Multimodal platforms often fail when they attempt to force LLMs, diffusion models, speech recognizers, neural TTS codecs, and video pipelines into a single monolithic inference engine.
- **Decision:** Define a lightweight `ModalityAdapter` trait in `miniai-core`. Each modality (`Llm`, `Vision`, `ImageGeneration`, `SpeechAsr`, `SpeechTts`, `Music`, `Video`) registers its own adapter with independent dependency graphs, process isolation, and health checks.
- **Consequences:** Adding or upgrading a vision or audio model runtime never risks breaking LLM inference, and crashed adapters can be restarted independently.

---

## ADR-003: CPU-First Execution with Dynamic Hardware Fallback
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** Developer laptops, CI runners, and edge devices frequently lack discrete NVIDIA GPUs.
- **Decision:** Every modality adapter and ML workflow must implement and verify a deterministic `AcceleratorBackend::Cpu` execution path first. Hardware accelerators (`Cuda`, `Metal`, `Vulkan`, `DirectMl`, `Rocm`, `CoreMl`, `Nnapi`) are dynamically probed and selected with automatic fallback to `Cpu`.
- **Consequences:** 100% of MiniAI functional tests run deterministically on standard CPU hosts and CI runners while taking advantage of accelerators when present.

---

## ADR-004: Default Local-Only Privacy Mode & Cryptographic Cloud Consent Gates
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** Users of local-first AI platforms require an ironclad guarantee that prompts, documents, and media never leave their device without explicit permission.
- **Decision:** `PrivacyPolicyEngine` defaults to `local_only_mode = true`. Cloud or distributed compute requires both disabling `local_only_mode` and supplying a non-expired, endpoint- and modality-scoped `CloudConsentToken`.
- **Consequences:** Accidental data exfiltration is impossible by construction and verified by unit and security tests.

---

## ADR-005: Strictly Append-Only Engineering Logs (`TODO.md` and `CHANGELOG.md`)
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** In-place editing of task lists and changelogs destroys historical auditability across multi-stage engineering projects.
- **Decision:** `TODO.md` and `CHANGELOG.md` are strictly append-only. Every task creation, status transition, completion, and stage release is appended as a new timestamped entry at EOF and validated automatically by `scripts/verify_stage0.py`.
- **Consequences:** Full immutable audit trail of project execution from Stage 0 through Stage 10.



---

## ADR-006: SQLite WAL Mode, Deterministic Migrations & Localhost-Only Authenticated IPC
- **Date:** `2026-11-05`
- **Status:** `ACCEPTED`
- **Context:** MiniAI requires robust local storage for workspace settings, model registry metadata, and audit logs, coupled with a responsive, secure communication channel between the Rust core supervisor and the Flutter UI shell.
- **Decision:**
  - Standardize on embedded SQLite using Write-Ahead Logging (`PRAGMA journal_mode = WAL`) and foreign key enforcement (`PRAGMA foreign_keys = ON`) in `crates/miniai-storage`.
  - Enforce deterministic database migrations verified by SHA-256 SQL checksums in `schema_migrations`, with atomic rollback and automated corruption quarantine recovery (`.corrupt.bak`).
  - Implement a typed asynchronous event bus (`tokio::sync::broadcast`) with bounded backpressure and late-subscriber replay ring buffers in `crates/miniai-server`.
  - Expose a localhost-only HTTP/SSE IPC server bound strictly to loopback (`127.0.0.1`), protected by per-session bearer nonce authentication (`Authorization: Bearer <nonce>`) on all endpoints.
- **Consequences:** Eliminates external network attack surfaces, prevents database locking under concurrent UI reads, ensures offline self-healing resilience, and provides real-time event streaming to the Flutter UI shell.


---

## ADR-007: Cryptographic Staging, Quarantine Isolation, Memory Admission Ceilings & Decoupled Modality Runtimes
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:** Managing multi-gigabyte neural network weights across 7 independent modalities requires rigorous protection against partial downloads, corrupted weights, unauthorized network exfiltration, system out-of-memory crashes, and cascading runtime failures.
- **Decision:**
  - Enforce isolated quarantine staging (`.part` for downloads, `.import.quarantine` for local file imports) in `crates/miniai-registry`. Streaming SHA-256 digests are computed during transfer, and files are atomically promoted to verified cache only upon exact digest validation. Corrupted or tampered files are purged immediately.
  - Reject remote downloads when `LocalOnlyPrivacyMode` is active while permitting fully offline local model imports.
  - Probe CPU SIMD instruction sets (`AVX2`, `AVX-512`, `NEON`) and host memory via `sysinfo` in `crates/miniai-core`. Enforce strict 85% available RAM and VRAM admission ceilings via `AdmissionGuard`.
  - Automatically fall back to `AcceleratorBackend::Cpu` when accelerator VRAM is insufficient, accelerators are absent, or `force_cpu_only` mode is set. Reject models with `MiniAiError::InsufficientMemory` if available system RAM is breached.
  - Orchestrate all 7 modalities via independent `ManagedAdapter` instances in `crates/miniai-runtime`, ensuring model switching or failure in one modality never degrades or crashes other active modalities.
- **Consequences:** Eliminates corruption risks in the model cache, guarantees system memory stability, prevents data leakage under local-only privacy mode, and ensures complete fault isolation across multimodal runtimes.

---

## ADR-008: Local LLM Engine Architecture, Hot Model Switching with Two-Phase Drop, Conversation Tree Traversal & Quantitative Benchmarking
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:** Stage 3 requires production-grade offline LLM text generation, multi-session conversation persistence, and quantitative performance benchmarking on resource-constrained consumer hardware without cloud dependencies. Specific challenges include:
  1. Preventing Out-Of-Memory (OOM) spikes during model switching when two large models might temporarily coexist in memory.
  2. Supporting conversation branching, in-place message editing, and Markdown/JSON export without complex relational schemas.
  3. Achieving responsive token streaming (<10ms cancellation responsiveness) without thread blocking or memory leaks.
  4. Delivering quantitative benchmark metrics (cold start, TTFT, tokens/sec, memory RSS) with statistical integrity.
- **Decision:**
  - Implement a dedicated `#![forbid(unsafe_code)]` LLM crate (`crates/miniai-llm`) with an autoregressive decoder, GGUF magic verification (`0x46554747`), subword BPE tokenization, and a configurable sampler supporting greedy sampling, temperature scaling, nucleus top-p filtering, and exponential frequency repetition penalties.
  - Enforce a strict **Two-Phase Drop Pattern** for hot model switching: `LlmEngine::switch_model` calls `unload_model()` and asserts `self.weights.is_none()` before opening or mapping the replacement model file. This guarantees that peak memory during model transitions never exceeds the size of a single loaded model.
  - Implement SQLite Migration 4 (`004_chat_messages.sql`) utilizing a self-referential `parent_id` column on `chat_messages`. Linear conversation history is reconstructed via upward recursive traversal (`get_message_branch`), while alternative completions are indexed via child queries (`get_child_branches`), eliminating separate tree-link tables.
  - Integrate a responsive Flutter Chat Workspace (`ChatWorkspaceScreen`) with real-time token streaming badges, live tokens/second telemetry, and immediate cooperative generation cancellation (`cancel_generation_button`).
  - Implement `LlmBenchmarkHarness` in `crates/miniai-llm/src/benchmark.rs` to measure real cold load times from disk, warm memory reloads, and per-iteration TTFT and throughput. Enforce strict warmup iteration exclusion from reported p50, p95, and mean statistics, persisting output in conformance with `miniai-evaluation-benchmarks` JSON schema.
- **Consequences:** Eliminates memory exhaustion during model transitions, guarantees non-destructive conversation branching and editing, maintains offline-first privacy guarantees with zero cloud leakage, and provides mathematically honest performance benchmarks.


---

## ADR-009: Dataset Curation, Parameter-Efficient Fine-Tuning (LoRA), Hinton Knowledge Distillation, and Cryptographic Checkpoint Provenance
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 4 requires a robust dataset curation pipeline, local-first parameter-efficient fine-tuning (PEFT), teacher-student knowledge distillation, cryptographic model provenance tracking, and an interactive Flutter training studio. Key engineering challenges include:
  1. Ingestion of untrusted datasets with variable schemas (`prompt`/`response`, `instruction`/`output`, `input_text`/`target_text`) without malformed UTF-8 characters or path traversal.
  2. Executing fine-tuning on consumer CPU hardware where full parameter updating of large models is intractable.
  3. Ensuring identity initialization in LoRA adapters ($h = W_0 x$ at $t=0$) so un-tuned adapters do not degrade base model weights before training commences.
  4. Teacher-student distillation loss balancing task accuracy with temperature-scaled soft distributions preserving $T^2$ gradient scale.
  5. Cryptographic reproducibility: every produced adapter checkpoint must record exact base model SHA-256, dataset SHA-256, random seed, hyperparameters, and convergence metrics.
  6. Seamless export and integration of fine-tuned weights into the Stage 2 Model Registry without manual reconfiguration.
- **Decision:**
  - Build `miniai_ml` in Python (`ml/src/miniai_ml/`) managed via `uv`, with PyTorch CPU-first execution (`torch>=2.15.0+cpu`).
  - Implement `miniai_ml/dataset.py`: multi-format loader (`.jsonl`, `.csv`), UTF-8 strict decoding, flexible field normalization, content-based deduplication, and deterministic PRNG splitting with cryptographic SHA-256 manifest verification.
  - Implement `miniai_ml/training.py`:
    - `LoraLinear`: low-rank matrix decomposition ($W_0 + \frac{\alpha}{r} B A$), initialized with $A \sim \text{Kaiming uniform}$ and $B = 0$ (identity initialization). In-place weight merging (`merge`) and unmerging (`unmerge`) preserving exact numerical equivalence.
    - `DistillationLoss`: $\mathcal{L}_{total} = \alpha_{task} \mathcal{L}_{task} + (1 - \alpha_{task}) T^2 \mathcal{L}_{KD}$ over softened teacher logits with $T^2$ gradient scale preservation.
    - `Trainer`: forward/backward optimization loop with AdamW, gradient accumulation, gradient norm clipping, periodic validation evaluation, atomic checkpoint persistence/resumption, and cooperative cancellation callback.
  - Implement `miniai_ml/provenance.py`: cryptographic manifest `checkpoint_provenance.json` tracking base model SHA-256, dataset SHA-256, random seed, device info, hyperparameters, and convergence metrics. Export catalog manifests directly compatible with Stage 2 `ModelManifest`.
  - Implement `TrainingStudioScreen` in Flutter (`ui/miniai_app`) as Tab 6 in `ShellScreen`: dataset split inspector, LoRA tuning form, live progress bar, loss sparkline, and base vs tuned evaluation comparison table.
- **Consequences:**
  Enables consumer hardware fine-tuning with zero cloud compute leakage, provides mathematical verification of adapter merging/unmerging, enforces end-to-end cryptographic traceability of trained artifacts, and seamlessly connects training outputs to the MiniAI model registry.


---

## ADR-010: Structure-Aware RAG, Hybrid Lexical/Dense Retrieval (BM25 + Dense + RRF), and Capability-Sandboxed Local Tools
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 5 requires local document and code indexing, hybrid retrieval with verifiable source citations, capability-sandboxed tool execution, and an interactive knowledge management UI:
  1. Preventing path traversal vulnerabilities, sensitive credential leaks (`.env`, `*.pem`, `id_rsa`, `*.db`), and binary file ingestion during workspace indexing.
  2. Structure-aware chunking for code and markdown that preserves semantic boundaries (headings, code blocks, syntax scopes) while tracking precise source locations (`file_path#Lstart-Lend`) for verifiable grounding.
  3. Incremental indexing to avoid re-chunking and re-embedding unchanged files across indexing passes using content-addressed SHA-256 caching.
  4. Hybrid retrieval combining lexical keyword precision (Okapi BM25 with IDF smoothing) and semantic dense embeddings with Reciprocal Rank Fusion (RRF, $k=60$) on CPU without heavy external vector database dependencies.
  5. Sandboxed local tool execution (`workspace_read_file`, `calculator`, `code_symbol_search`, `rag_query`) with strict capability permissions, resource bounds (timeouts, byte caps), and non-bypassable Local-Only Privacy Mode blocking outbound network tools.
  6. Cross-platform UI integration with knowledge inventory, retrieval tester, and interactive citation inspector in chat bubbles.
- **Decision:**
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-rag` with clean separation of concerns:
    - `security.rs`: Canonical path validation preventing directory traversal outside workspace root with Windows UNC (`\\?\`) prefix stripping, sensitive file exclusion (`.env`, `*.pem`, `id_rsa`, `*.db`, keys), and binary byte inspection.
    - `chunker.rs`: Structure-aware text and code chunker preserving headings, code fence blocks, and syntax scopes, tagging each chunk with exact `file_path#Lstart-Lend` citation tags.
    - `dedup.rs`: Content-addressed chunk deduplication tracking unique SHA-256 digests.
    - `pipeline.rs`: Incremental `IngestionPipeline` with manifest caching (`IngestionManifest`) skipping unchanged files on subsequent runs.
    - `bm25.rs`: Okapi BM25 lexical index with Robertson-Spärck Jones IDF smoothing ($k_1=1.2, b=0.75$).
    - `embedding.rs`: Deterministic local 128-dimensional L2-normalized dense embeddings and vector cosine similarity search.
    - `retrieval.rs`: `HybridSearchEngine` fusing lexical and dense search results via Reciprocal Rank Fusion ($1/(k + rank)$ with $k=60$) with grounded citation formatting.
    - `tools.rs`: Capability security model (`ToolCapability::WorkspaceRead`, `ToolCapability::Compute`, `ToolCapability::CodeSearch`, `ToolCapability::Network`) with fine-grained access control in `ToolExecutionContext`. Built-in tools: `WorkspaceFileReadTool`, `CalculatorTool` (safe AST math parser on CPU), `CodeSymbolSearchTool`, and `RagQueryTool`. `ToolSandboxRunner` coordinates capability checking, execution timeout guard, output byte cap truncation, and strictly blocks network access when `LocalOnlyPrivacyMode` is active.
    - Extended Flutter UI in `ui/miniai_app`: `KnowledgeBaseScreen` (Tab 7 'Knowledge' in `ShellScreen`) with index health stats, document inventory data table with SHA-256 digests, RAG grounding toggle, interactive query tester with RRF/Dense/BM25 score chips, and source modal inspection. Added clickable citation chips to `ChatWorkspaceScreen` assistant message bubbles.
- **Consequences:**
  Guarantees 100% offline document and code grounding, eliminates hallucination risks through verifiable source line citations, prevents unauthorized file system traversal or network leakage, and maintains sub-millisecond local hybrid search throughput.


---

## ADR-011: Decoupled Multimodal Architecture: Vision Understanding, Iterative CPU Image Synthesis & Canvas Studio
- **Date:** `2026-11-06`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 6 introduces visual modality capabilities (`Modality::Vision` and `Modality::ImageGeneration`) requiring independent pluggable architecture, strict local-first privacy, robust media safety defenses, and CPU-first execution:
  1. Image sanitization against decompression bombs, truncated headers, corrupt payloads, and oversized inputs (limiting dimensions $\le 4096\times 4096$ and file sizes $\le 25\text{ MB}$).
  2. CPU-first visual processing: bilinear tensor resampling to standard model resolutions ($224\times 224$), ImageNet mean/std normalization, localized OCR text and bounding-box detection, automated scene descriptions, and grounded visual question answering.
  3. Decoupled image generation and editing: Text-to-Image synthesis, Image-to-Image refinement via forward diffusion noise scheduling, and localized binary mask inpainting preserving unmasked base pixels exactly.
  4. Native lossless PNG encoding and decoding supporting standard Deflate uncompressed blocks and Adler32/CRC32 checksums, with cryptographic provenance chunk embedding (`tEXt` chunks preserving model ID, prompt, negative prompt, seed, steps, guidance scale, dimensions, timestamp, and SHA-256 fingerprint).
  5. Cooperative generation cancellation via cancellation tokens and callback abort flags.
  6. Interactive Flutter Canvas Studio with custom inpainting mask drawing, brush/eraser modes, real-time generation progress telemetry with per-step loss reporting, and a local gallery with provenance inspection modals.
- **Decision:**
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-vision` implementing `ModalityAdapter` (`Modality::Vision`):
    - `sanitizer.rs`: `ImageSanitizer` enforcing magic-byte inspection (PNG, JPEG, WebP VP8/VP8L/VP8X), dimension bounds ($\le 4096\times 4096$), max file size ($\le 25\text{ MB}$), decompression bomb mitigation, and SHA-256 calculation.
    - `tensor.rs`: `ImageTensor` implementing bilinear resampling, value scaling, ImageNet mean/std normalization, luminance, and spatial gradient edge energy computation.
    - `engine.rs`: `VisionEngine` delivering CPU-first OCR detection with localized bounding boxes (`OcrResult`), automated scene description and captioning (`VisualDescription`), and grounded Visual Question Answering (`VisualQaAnswer`).
    - `adapter.rs`: `VisionAdapter` implementing `ModalityAdapter` (`Modality::Vision`), CPU-first execution (`AcceleratorBackend::Cpu`), weight loading with SHA-256 digest validation, and live health check reporting.
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-imagegen` implementing `ModalityAdapter` (`Modality::ImageGeneration`):
    - `types.rs`: `ImageGenConfig` with multiple-of-64 dimension bounds, `GenerationMode` (`TextToImage`, `ImageToImage`, `Inpainting`), `StepProgress`, and `ImageProvenance`.
    - `png.rs`: `PngEncoder` and `PngDecoder` implementing standard PNG headers, Deflate uncompressed block streams, Adler32 and CRC32 checksums, and `tEXt` cryptographic provenance chunk injection and retrieval.
    - `engine.rs`: `ImageGenEngine` with deterministic Xorshift64 PRNG, text conditioning, iterative scheduled denoising loop with step progress callback telemetry, forward diffusion noise addition for Image-to-Image refinement, and binary mask blending for inpainting preserving unmasked base pixels exactly.
    - `adapter.rs`: `ImageGenAdapter` implementing `ModalityAdapter` (`Modality::ImageGeneration`) on CPU.
  - Extend Flutter UI under `ui/miniai_app`:
    - `system_contracts.dart`: `OcrBoxContract`, `VisionAnalysisResultContract`, `GenerationModeContract`, `ImageGenTaskConfig`, and `GeneratedImageItem`.
    - `platform_controller.dart`: Vision image upload and format sanitizer, SHA-256 fingerprinting, OCR detection, grounded Visual QA, inpainting mask brush/eraser controls, image generation parameters, cooperative cancellation, and local canvas gallery.
    - `vision_canvas_screen.dart`: Registered as Tab 8 ('Canvas') in `ShellScreen` with dual tabs: Vision Understanding (viewer, format chips, OCR detections list, scene description tags, grounded VQA) and Canvas Studio & Gallery (parameter controls, custom inpainting mask canvas, step progress banner with loss badge and cancel button, and local gallery with cryptographic provenance modal).
- **Consequences:**
  Eliminates cross-modality failure cascades between text and visual pipelines, provides zero-mock visual inference and image generation on commodity CPUs without GPU requirements, strictly protects user privacy under Local-Only Privacy Mode, and ensures tamper-evident cryptographic provenance for all generated image assets.


---

## ADR-012: Offline Speech Recognition (ASR), Formant Acoustic TTS, Polyphonic Music Generation & Voice Conversation Loop Architecture
- **Date:** `2026-11-07`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 7 delivers audio, speech, and musical modalities (`Modality::SpeechAsr`, `Modality::SpeechTts`, and `Modality::Music`) requiring strict offline operation, commodity CPU execution (`AcceleratorBackend::Cpu`), robust input validation against malformed audio, zero-mock UI controls, and a unified offline voice conversation loop:
  1. Audio format sanitization: validate container headers (WAV PCM/float, FLAC, OGG), channel limits ($1..=8$), sample rate limits ($8\text{ kHz}..=192\text{ kHz}$), max duration ($\le 600\text{s}$), file size ($\le 25\text{ MB}$), and cryptographic SHA-256 fingerprinting.
  2. Audio preprocessing & resampler: multi-channel to mono downmixing via arithmetic averaging, band-limited linear interpolation resampling to standard $16\text{ kHz}$ mono float32, and RMS loudness normalization.
  3. Offline Voice Activity Detection (VAD): 30ms short-time energy (STE), zero-crossing rate (ZCR), adaptive noise floor estimation with continuous speech handling, and hangover smoothing.
  4. Speech recognition (ASR): deterministic acoustic subword mapping with segment and word-level timestamps (`start_secs`, `end_secs`, `text`, `confidence`).
  5. Lossless RIFF/WAVE encoding: 16-bit signed PCM mono and stereo with sample clipping clamping.
  6. Neural Text-to-Speech (TTS): safety limits ($\le 4096$ chars per segment), word expansion for numbers (up to 999,999) and symbols, acoustic formant synthesis featuring 6 preset voice profiles (`Alloy`, `Echo`, `Fable`, `Onyx`, `Nova`, `Shimmer`), 3-formant resonance model ($F_1, F_2, F_3$), harmonic overtone series, unvoiced noise synthesis for fricatives and plosive bursts, continuous phase accumulator, ADSR envelope shaping, and soft-saturation limiter.
  7. Offline Voice Conversation Loop: orchestration pipeline linking `SpeechAsrAdapter` $\to$ `LlmEngine` $\to$ `SpeechTtsAdapter` with end-to-end turnaround latency profiling (`asr_latency_ms`, `llm_ttft_ms`, `llm_total_latency_ms`, `tts_latency_ms`, `total_turnaround_ms`) and cooperative cancellation.
  8. Offline Polyphonic Music Generation: musical scales (`C Major`, `A Minor`, `G Major`, `E Minor`, `D Dorian`), 4-track acoustic synthesis (bassline, stereo harmony pad triads, melody lead arpeggios with vibrato, and rhythm kick/snare/hi-hat percussion), and stereo soundstage spatializer.
  9. Flutter Sound Studio: dual-tab interface in `SoundStudioScreen` (Tab 9 in `ShellScreen` across desktop `NavigationRail` and mobile `NavigationBar`) supporting ASR transcription with confidence and timestamped segments, Neural TTS voice profiles and sliders, live voice conversation loop timeline with latency telemetry chips, and Music Studio with soundstage controls and local music library.
- **Decision:**
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-speech-asr` implementing `ModalityAdapter` (`Modality::SpeechAsr`):
    - `sanitizer.rs`: `AudioSanitizer` validating WAV, FLAC, and OGG formats with boundary defenses and SHA-256 calculation.
    - `resampler.rs`: `AudioResampler` performing multi-channel downmixing, linear interpolation resampling to 16 kHz mono, and RMS normalization.
    - `vad.rs`: `VadEngine` with frame-based energy, zero-crossing rate, adaptive noise floor estimation, and hangover smoothing.
    - `engine.rs`: `SpeechAsrEngine` producing timestamped phonetic subwords and speech segments.
    - `adapter.rs`: `SpeechAsrAdapter` conforming to `ModalityAdapter` on CPU with SHA-256 weights verification.
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-speech-tts` implementing `ModalityAdapter` (`Modality::SpeechTts`):
    - `wav.rs`: `WavEncoder` producing compliant 16-bit signed PCM WAV containers.
    - `normalizer.rs`: `TextNormalizer` enforcing 4096-char bounds, number/symbol expansion, sentence chunking, and phonetic tokenization.
    - `synthesizer.rs`: `SpeechTtsEngine` with 6 voice profiles, formant resonances, harmonic overtones, ADSR envelopes, soft limiter, and deterministic PRNG reproducibility.
    - `adapter.rs`: `SpeechTtsAdapter` conforming to `ModalityAdapter` on CPU.
    - `voice_loop.rs`: `VoiceConversationLoop` coordinating offline voice turns (`ASR -> LLM -> TTS`) with detailed latency telemetry.
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-music` implementing `ModalityAdapter` (`Modality::Music`):
    - `types.rs`: `MusicalScale` providing modal frequencies and chord triads, `MusicGenConfig`, and `MusicTrackOutput`.
    - `wav.rs`: `WavEncoder` supporting mono and stereo 16-bit PCM WAV encoding.
    - `engine.rs`: `MusicGenEngine` polyphonic 4-track acoustic synthesizer with stereo spatial panning and soft limiter.
    - `adapter.rs`: `MusicAdapter` conforming to `ModalityAdapter` on CPU.
  - Extend Flutter UI under `ui/miniai_app`:
    - `system_contracts.dart`: Contracts for Speech ASR, TTS, Voice Loop turns, and Music Generation.
    - `platform_controller.dart`: State and operations for ASR audio ingestion, TTS synthesis, offline voice loop turns, and music composition.
    - `sound_studio_screen.dart`: Tab 9 in `ShellScreen` with dual tabs for Speech & Voice Loop and Music Studio & Soundstage.
- **Consequences:**
  Establishes complete local-first speech and audio capability on CPU without external cloud dependencies, eliminates cloud audio latency and eavesdropping risks, achieves sub-second turnarounds for voice interactions on standard hardware, and ensures zero mock policy across all audio modalities.


---

## ADR-013: Video Understanding, Generative Motion Synthesis, and Composable Multimodal DAG Orchestration Architecture
- **Date:** `2026-11-07`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 8 delivers Video Understanding, Generative Video Motion Synthesis, and Composable Multimodal DAG Pipeline Orchestration (`Modality::Video`), completing all 7 primary modalities of MiniAI under universal CPU-first baseline execution (`AcceleratorBackend::Cpu`) and Local-Only Privacy Mode:
  1. Video Container Sanitization & Security Defenses: Inspection of binary box headers for MP4 (`ftyp` atoms: `isom`, `avc1`, `hvc1`, `av01`, `vp09`), WebM (EBML master element `0x1A45DFA3` with `VP8`, `VP9`, `AV1`, `Vorbis`, `Opus`), and `RAWV` frame streams; validation of strict file size bounds ($\le 100\text{ MB}$), duration bounds ($\le 300\text{s}$), spatial dimensions ($\le 1920\times 1080$), and frame rate bounds ($1..=60\text{ fps}$); calculation of deterministic streaming SHA-256 digest to mitigate zip bombs, malformed atoms, and resource exhaustion.
  2. Bounded Frame Memory Management: CPU-based video sampling can easily exhaust system RAM (e.g., 24 frames of 1080p RGBA requires >190 MB). To ensure stability on constrained host environments, a shared bounded frame buffer pool (`FrameBufferPool`) strictly caps resident frame memory (default 64 MB), enforces zero-leak buffer recycling on frame drop, and provides automatic memory shrinkage.
  3. RAII Temporary Frame Workspaces: Atomic creation of isolated temporary frame cache directories (`AtomicWorkspaceManager`) ensuring automatic filesystem cleanup on drop or cancellation without leaking orphan temporary files.
  4. Frame Extraction & Adaptive Keyframe Sampling: Multiple sampling strategies including uniform interval sampling, uniform duration sampling, scene boundary delta detection via downsampled 1024-byte frame signatures and normalized L1 pixel difference, and adaptive dynamic keyframe extraction recycling candidate frame buffers back to the memory pool.
  5. Audio Demuxing: Demuxing embedded video audio streams into standard 16-bit PCM WAV containers directly verified against `AudioSanitizer`.
  6. Temporal Scene Segmentation & Grounded Video QA: Classification of temporal scenes into 5 archetypes (`EstablishingShot`, `DynamicAction`, `DialogueCloseUp`, `LandscapePan`, `InteriorDetail`), camera and subject motion detection (`CameraPan`, `RapidMotion`, `SceneTransition`, `StaticHold`), narrative video summary synthesis, and millisecond-exact grounded temporal VQA (`start_ms`, `end_ms`, supporting scene and keyframes).
  7. Local Video Synthesis: Text-to-video generative keyframe animation, 8 camera motion profiles (`Static`, `PanLeft`, `PanRight`, `ZoomIn`, `ZoomOut`, `TiltUp`, `TiltDown`, `Orbit`), image-to-video motion interpolation, two-keyframe morph interpolation, synchronized audio track generation (verified via `AudioSanitizer`), and `RAWV` video packaging.
  8. Composable Multimodal DAG Workflows: A directed acyclic graph (`MediaWorkflowGraph`) chaining heterogeneous modality nodes (`LlmPrompt`, `VisionAnalyze`, `ImageGen`, `SpeechTts`, `SpeechAsr`, `MusicGen`, `VideoSynthesis`, `VideoAnalysis`, `AudioVideoMux`, `Custom`); topological sorting via Kahn's algorithm with cycle detection (`VideoError::WorkflowCycleDetected`); and an asynchronous execution engine (`WorkflowExecutor`) with typed data passing (`WorkflowData`), per-node execution status and duration telemetry, and cooperative cancellation.
  9. Flutter Video Studio & Workflow Monitor: Dual-tab interface in `VideoStudioScreen` (Tab 10 in `ShellScreen` across desktop `NavigationRail` and mobile `NavigationBar`):
     - Tab 1: Video Timeline & Scene QA (video player card, millisecond scrubber, play/pause controls, prev/next keyframe buttons, keyframe gallery strip, temporal scene segments card with action chips, and grounded temporal video QA).
     - Tab 2: Video Synthesis & Multimodal DAG Workflows (generative video synthesis form with prompt, camera motion profile, duration slider, resolution selector, and DAG workflow orchestrator monitor showing real-time multi-modality node execution).
- **Decision:**
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-video` implementing `ModalityAdapter` (`Modality::Video`):
    - `sanitizer.rs`: `VideoSanitizer` inspecting MP4, WebM, and RAWV headers, validating bounds, and computing SHA-256 digests.
    - `buffer_pool.rs`: `FrameBufferPool` and `SharedFrameBufferPool` providing memory-bounded buffer recycling with zero pool leakage.
    - `extractor.rs`: `AtomicWorkspaceManager` and `FrameExtractor` supporting uniform interval/duration, scene boundary change detection, and adaptive dynamic keyframes.
    - `demuxer.rs`: `AudioDemuxer` extracting standard WAV PCM audio streams from multimedia containers.
    - `engine.rs`: `VideoEngine` performing scene classification, motion classification, narrative summary, and grounded temporal VQA.
    - `synthesis.rs`: `VideoSynthesisEngine` providing local generative text-to-video animation, camera motion profiles, image-to-video motion interpolation, and synchronized audio generation.
    - `workflow.rs`: `MediaWorkflowGraph` and `WorkflowExecutor` providing composable multimodal DAG execution with topological sorting, cycle detection, typed data passing, and duration telemetry.
    - `adapter.rs`: `VideoAdapter` implementing `ModalityAdapter` (`Modality::Video`) on CPU, model weights loading with SHA-256 digest validation, and extended operations for `synthesize_video` and `execute_workflow`.
  - Extend Flutter UI under `ui/miniai_app`:
    - `system_contracts.dart`: Contracts for `VideoMetadataContract`, `VideoFrameContract`, `SceneSegmentContract`, `ActionDetectionContract`, `VideoAnalysisContract`, `TemporalQaResultContract`, `VideoSynthesisConfigContract`, `SynthesizedVideoContract`, `WorkflowNodeContract`, and `WorkflowExecutionResultContract`.
    - `platform_controller.dart`: State and zero-mock methods for video analysis, timeline scrubbing, temporal video QA, prompt/image video synthesis, and multimodal media DAG workflow execution.
    - `video_studio_screen.dart`: Tab 10 in `ShellScreen` with dual tabs for Video Timeline & Scene QA and Video Synthesis & Multimodal DAG Workflows.
- **Consequences:**
  Completes the 7-modality architecture of MiniAI without external cloud dependencies or GPU hardware lock-in, eliminates host out-of-memory risks during high-resolution frame extraction via bounded frame memory pooling, provides mathematically verified topological DAG execution across text, vision, audio, music, and video pipelines, and guarantees local-first privacy protection with zero mock implementations across desktop and mobile platforms.

---

## ADR-014: Scoped Cryptographic Cloud Consent Governance, mTLS Distributed Compute Dispatch, and Seamless Local CPU Fallback
- **Date:** `2026-11-07`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 9 delivers secure distributed compute and hybrid cloud GPU execution (`crates/miniai-cloud`) while uncompromisingly upholding MiniAI's core architectural tenets: local-first privacy, zero secret leakage, fail-closed access control, and deterministic CPU-first execution (`AcceleratorBackend::Cpu`):
  1. Consent Governance & Scoped Cryptographic Tokens: Default `LocalOnlyPrivacyMode = true` strictly blocks all outbound network transmissions. Explicit cloud compute requires a cryptographically validated `CloudConsentToken` specifying token ID, target HTTPS endpoint, modality binding, maximum payload byte limit ($\le 100\text{ MB}$), validity duration ($\le 24\text{ hours}$), and mandatory user acknowledgement. Loopback endpoints (`127.0.0.1`, `::1`, `localhost`) are strictly forbidden in production to prevent SSRF and local sandbox bypasses.
  2. Append-Only Immutable Audit Ledger: Every consent event (`Granted`, `Verified`, `Revoked`, `Expired`, `Rejected`, `PayloadDispatched`) is persisted to an append-only SQLite audit table with zero update/delete operations, monotonic sequence IDs, and rich query filters (`query_by_token`, `query_by_action`, `query_by_modality`, `query_by_time_range`).
  3. Pre-Flight Verification Gate & Mid-Flight Cancellation: `ConsentManager::verify_and_authorize` must pass prior to payload serialization or network transmission; if privacy mode is active, consent is revoked, expired, mismatched, or quota exhausted, the dispatch fails closed locally with zero egress. Mid-flight cooperative cancellation token checks abort pending remote operations.
  4. Encrypted Remote Compute Dispatcher & mTLS: `TlsConfig` enforces mandatory TLS 1.3 policy and optional mutual TLS (mTLS) with client certificate and private key validation against CA roots. `RemoteDispatcher` submits scoped `ComputeTaskRequest` envelopes and accounts consumed bytes in the ledger.
  5. Automatic Local CPU Fallback: `FallbackCoordinator` intercepts remote worker network failures, timeouts, 5xx server errors, or mid-flight revocations, transparently executing the workload on local CPU runtime adapters (`local_cpu_adapter`) with zero data loss while recording fallback audit telemetry.
  6. Flutter Consent Inspector & Emergency Kill-Switch: Tab 11 ('Cloud') in `ShellScreen` provides a prominent instant Emergency Kill-Switch button, a scoped token issuance form with user acknowledgement gating, active token manager with countdown timers, a hybrid dispatch simulator testing remote vs local fallback, and an immutable audit ledger table.
- **Decision:**
  - Create `#![forbid(unsafe_code)]` crate `crates/miniai-cloud`:
    - `error.rs`: `CloudError` domain failure variants.
    - `types.rs`: Core data types (`ConsentAction`, `ConsentStatus`, `ConsentScope`, `ConsentAuditRecord`, `ManagedConsentToken`).
    - `token.rs`: `TokenValidator` with defense-in-depth boundary checks and SHA-256 token fingerprinting.
    - `ledger.rs`: `ConsentLedger` append-only audit repository backed by transactional SQLite via `StorageEngine`.
    - `manager.rs`: `ConsentManager` thread-safe coordinator orchestrating issuance, verification, revocation, and quota tracking.
    - `dispatcher.rs`: `TlsConfig`, `ComputeTaskRequest`, `ComputeTaskResponse`, `RemoteWorkerTransport` trait, `SimulatedRemoteTransport`, and `RemoteDispatcher`.
    - `fallback.rs`: `LocalFallbackHandler` trait, `DefaultLocalFallbackHandler`, and `FallbackCoordinator`.
  - Extend Flutter UI under `ui/miniai_app`:
    - `system_contracts.dart`: Contracts for `ConsentActionContract`, `ConsentAuditRecordContract`, `ManagedConsentTokenContract`, `ComputeTaskRequestContract`, and `ComputeTaskResponseContract`.
    - `platform_controller.dart`: State and zero-mock methods for scoped token issuance, revocation, emergency kill-switch, and hybrid compute dispatch with local CPU fallback.
    - `consent_inspector_screen.dart`: Tab 11 in `ShellScreen` featuring emergency kill-switch, scoped consent request dialog, active token manager, interactive hybrid dispatch simulator, and immutable audit ledger viewer.
- **Consequences:**
  Enables scalable cloud GPU acceleration without compromising local privacy or user sovereignty; guarantees zero egress in local-only privacy mode; prevents SSRF and secret leaks; eliminates task dropouts through transparent local CPU fallback; and provides verifiable append-only auditing for enterprise compliance.


---

## ADR-015: Cross-Platform Hardening, Release Certification, and CycloneDX Software Bill of Materials (SBOM) for MiniAI v1.0.0
- **Date:** `2026-11-07`
- **Status:** `ACCEPTED`
- **Context:**
  Stage 10 delivers full-system production hardening, property-based fuzzing, SQLite crash resilience, reproducible cross-platform packaging, CycloneDX v1.5 Software Bill of Materials (SBOM) generation, SHA-256 cryptographic release manifests, and final Release Certification (`v1.0.0`) across all 5 target operating systems (Windows, Linux, macOS, Android, iOS):
  1. System-Wide Stress & Property-Based Fuzzing: Fuzzing harnesses in `tests/fuzz_and_stress_tests.rs` verify all 7 modality adapters (LLM, Vision, ImageGen, Speech ASR, Speech TTS, Music, Video) against malformed payloads, out-of-bounds parameter configurations, prompt injection strings, runaway tokens, decompression bombs, corrupted audio/video container headers, and memory buffer pool exhaustion without crashing or leaking memory.
  2. Transactional SQLite Crash Recovery: `crates/miniai-storage/tests/recovery_tests.rs` tests zero-byte file initialization, unclosed transaction rollbacks, foreign key constraint preservation, corrupt database page quarantine, and concurrent multi-threaded stress under WAL mode.
  3. Zero-Vulnerability Security Audit: Dependency scanning via `cargo audit` ensures zero known CVEs across all 89 Rust workspace dependencies, and repository governance auditing ensures zero private key blocks or secret patterns exist in source or documentation.
  4. Reproducible Cross-Platform Packaging: `scripts/package_release.py` generates deterministic release bundles for Windows (`x86_64`), Linux (`x86_64`), macOS (`universal`), Android (`APK`/`AAB`), and iOS (`unsigned IPA`).
  5. CycloneDX v1.5 SBOM & SHA-256 Manifests: Produces machine-readable CycloneDX v1.5 JSON (`bom.json`) and XML (`bom.xml`) inventorying all 14 workspace crates, Python ML tooling, and Flutter UI components with licenses and SHA-256 digests. Generates master `SHA256SUMS.txt` with cryptographic integrity verified via `--verify`.
  6. Final Multi-OS CI/CD Release Pipeline: `.github/workflows/release.yml` automates multi-platform build verification across `windows-latest`, `ubuntu-latest`, and `macos-latest`, building Flutter targets, generating SBOMs, and uploading certified release assets.
- **Decision:**
  - Standardize cross-platform release generation through `scripts/package_release.py`.
  - Enforce CycloneDX v1.5 format for all release bill-of-materials deliverables.
  - Require SHA-256 cryptographic digest manifests for 100% of released packages and assets.
  - Require unanimous `PASS` sign-offs from `independent-qa-auditor` and `arch-security-reviewer` before certifying `v1.0.0`.
- **Consequences:**
  Guarantees enterprise-grade release provenance, supply chain transparency, reproducible builds across desktop and mobile platforms, zero known security vulnerabilities, and verifiable compliance with the MiniAI Global Engineering Contract.
