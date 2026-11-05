# Stage 6 Technical Execution Prompt: Vision, Image Understanding, Generation & Editing

## 1. Objective & Dependencies
- **Objective:** Deliver independent Vision Understanding (`Modality::Vision`) and Image Generation/Inpainting (`Modality::ImageGeneration`) runtime adapters and the Flutter Vision & Canvas Studio.
- **Dependencies:** Stage 5 Gate Verified.
- **Active Skills:** `miniai-multimodal`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-flutter-ui`, `miniai-security-privacy`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 6.1: Vision Understanding Runtime Adapter (`crates/miniai-vision`)**
  - *Subphase 6.1.1:* Image magic-byte & dimension sanitizer (`<= 4096x4096`, `<= 25 MB`), tensor normalization, and CPU-first visual QA / captioning / OCR adapter.
- **Phase 6.2: Image Generation & Editing Runtime Adapter (`crates/miniai-imagegen`)**
  - *Subphase 6.2.1:* Text-to-image synthesis, image-to-image refinement, and mask-based inpainting with deterministic seeds, step progress events, and PNG provenance metadata.
- **Phase 6.3: Flutter Vision & Image Canvas Studio**
  - *Subphase 6.3.1:* Interactive image upload/inspection, inpainting brush mask editor, generation parameter controls, and local gallery persistence.

## 3. Security & Edge Cases
- Reject decompression bombs, truncated image headers, and non-image MIME payloads.
- Keep `Modality::Vision` and `Modality::ImageGeneration` completely decoupled from `Modality::Llm`.

## 4. Acceptance Tests & Exit Criteria
- Vision analysis and image synthesis pass deterministic CPU tests.
- Oversized and malformed image files are safely rejected.
- >= 90% coverage across `miniai-vision` and `miniai-imagegen`.

