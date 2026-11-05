# Offline-First Verification Matrix

| Operation | `LocalOnlyPrivacyMode = true` | `LocalOnlyPrivacyMode = false` (No Cloud Consent) | `LocalOnlyPrivacyMode = false` (Valid Cloud Consent) |
| :--- | :--- | :--- | :--- |
| Local CPU/GPU Inference | Allowed | Allowed | Allowed |
| Local Dataset & RAG Indexing | Allowed | Allowed | Allowed |
| Outbound Telemetry | Blocked (Never Exists) | Blocked (Never Exists) | Blocked (Never Exists) |
| Hybrid Cloud GPU Execution | Blocked (`PrivacyModeBlocked`) | Blocked (`ConsentRequired`) | Allowed (Scoped to Token) |

