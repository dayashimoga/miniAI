# Cross-Language Boundary Contracts Reference

## 1. Canonical JSON Schema Rules
- Field names across JSON IPC boundaries use `snake_case`.
- Timestamps use ISO-8601 UTC strings (`YYYY-MM-DDTHH:MM:SSZ`).
- Digests use lowercase hex-encoded SHA-256 (`64` hex characters).
- Enum variants serialize as `snake_case` strings (`"llm"`, `"vision"`, `"image_generation"`, `"speech_asr"`, `"speech_tts"`, `"music"`, `"video"`).

## 2. Error Envelope Contract
```json
{
  "code": "PRIVACY_MODE_BLOCKED",
  "message": "Outbound cloud execution is prohibited while Local-Only Privacy Mode is active.",
  "recoverable": true,
  "remediation": "Disable Local-Only Privacy Mode and grant an explicit CloudConsentToken."
}
```

