---
name: miniai-offline-first
description: >-
  Guides offline-first architecture, Local-Only Privacy Mode enforcement, local artifact caching,
  and network-disconnected resilience across MiniAI (Stages 0–10). Activate whenever implementing
  storage, model loading, UI state, or network-aware workflows.
---

# MiniAI Offline-First & Local Privacy Skill

Use this skill to ensure every feature works offline and respects `LocalOnlyPrivacyMode`.

## Offline-First Guarantees
1. **Zero Network Dependency for Core Workflows**: Booting the app, browsing local models/datasets, running local inference across all modalities, and viewing documentation/benchmarks must work with zero internet connectivity.
2. **Hard Network Gate in `LocalOnlyPrivacyMode`**:
   - `PrivacyPolicyEngine::authorize_outbound` must reject any external network request when `local_only_mode == true`.
3. **Resumable & Verifiable Local Cache**:
   - Local artifacts are indexed by content-addressed SHA-256 hashes in the local registry.

## Workflow Steps
1. Read [`references/offline_verification_matrix.md`](./references/offline_verification_matrix.md).
2. Route all outbound network/cloud operations through `PrivacyPolicyEngine`.
3. Write explicit offline tests verifying that local operations succeed while outbound calls fail closed when `LocalOnlyPrivacyMode` is active.

## Verification
- Run `cargo test`, `flutter test`, and `uv run pytest` offline policy test suites.

