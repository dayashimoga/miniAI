# Multimodal Media Input & Output Limits Reference

| Modality | Accepted Input Formats | Default Safety Bounds | Output Formats |
| :--- | :--- | :--- | :--- |
| Vision | PNG, JPEG, WebP | Max resolution: `4096x4096`, Max size: `25 MB` | Structured JSON / Markdown |
| Image Gen/Edit | Text prompt + optional PNG mask | Resolution step multiple of `64` (`256`–`2048`) | Lossless PNG with provenance metadata |
| Speech ASR | WAV (PCM 16-bit), FLAC, OGG | Sample rate normalized to `16 kHz` mono | Timestamped transcript JSON |
| Speech TTS | UTF-8 text | Max chunk: `4096` chars per segment | WAV (`22.05 kHz` / `24 kHz` PCM) |
| Music | Text prompt + duration (`1s`–`120s`) | Memory-bounded chunked generation | WAV / FLAC |
| Video | MP4, WebM | Frame sampling cap configurable (`1`–`30` fps) | MP4 + Frame Analysis JSON |

