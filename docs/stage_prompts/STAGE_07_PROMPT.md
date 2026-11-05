# Stage 7 Technical Execution Prompt: Speech, Audio, TTS, ASR, Voice & Music

## 1. Objective & Dependencies
- **Objective:** Deliver independent Speech ASR (`Modality::SpeechAsr`), Neural TTS (`Modality::SpeechTts`), real-time offline voice loop, and Music Generation (`Modality::Music`) adapters and the Flutter Audio & Music Studio.
- **Dependencies:** Stage 6 Gate Verified.
- **Active Skills:** `miniai-multimodal`, `miniai-model-runtimes`, `miniai-hardware-optimization`, `miniai-flutter-ui`, `miniai-testing-qa`.

## 2. Phases & Subphases
- **Phase 7.1: Offline Speech Recognition (`crates/miniai-speech-asr`)**
  - *Subphase 7.1.1:* WAV/FLAC/OGG PCM resampler (`16 kHz` mono), Voice Activity Detection (VAD), and timestamped Whisper/Conformer ASR adapter.
- **Phase 7.2: Neural Text-to-Speech (`crates/miniai-speech-tts`) & Voice Loop**
  - *Subphase 7.2.1:* CPU-first neural TTS synthesis producing valid WAV PCM audio streams and low-latency offline voice conversation orchestration (`ASR -> LLM -> TTS`).
- **Phase 7.3: Music Generation (`crates/miniai-music`) & Audio Studio UI**
  - *Subphase 7.3.1:* Prompt-driven music/audio synthesis adapter with duration bounds and Flutter waveform recorder, transcript viewer, and audio player.

## 3. Security & Edge Cases
- Validate WAV/FLAC headers and enforce maximum duration/file-size limits to prevent memory exhaustion.
- Ensure microphone capture requires explicit user toggle in the UI.

## 4. Acceptance Tests & Exit Criteria
- Offline ASR transcribes reference WAV fixtures accurately on CPU.
- Offline TTS generates valid, non-silent WAV PCM output.
- >= 90% coverage across `miniai-speech-asr`, `miniai-speech-tts`, and `miniai-music`.

