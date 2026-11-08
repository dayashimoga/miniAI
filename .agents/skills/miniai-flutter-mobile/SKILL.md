---
name: miniai-flutter-mobile
description: >-
  Guides mobile Flutter engineering for MiniAI across Android (APK/AAB) and iOS (Runner/IPA).
  Activate when tuning touch interactions, back navigation, memory budgeting under mobile constraints,
  or build automation for mobile targets.
---

# MiniAI Flutter Mobile Engineering Skill

Use this skill when building, testing, or configuring mobile targets in `ui/miniai_app/`.

## Mobile Constraints & Best Practices
1. **Strict Mobile Memory Budgets**:
   - Mobile devices operate under strict low-memory killer (LMK) rules.
   - Enforce AdmissionGuard memory ceiling before attempting model downloads or runtime loading.
   - Default mobile models to ultra-compact quantized variants: `smollm2-135m-instruct-q4` (105MB) or `whisper-tiny-en` (77MB).
2. **Touch & Navigation Ergonomics**:
   - Use bottom `NavigationBar` with at least $48\times 48\text{dp}$ touch targets for all action chips and buttons.
   - Support Android back-gesture / back-button handling gracefully without losing unsaved prompts.
3. **Background Download Resilience**:
   - Provide visual progress indicators (`LinearProgressIndicator` + byte count + percentage) for model downloads.
   - Ensure downloads can resume or cleanly cancel without corrupting local cache.
4. **Mobile Packaging & Signing**:
   - Android: `flutter build apk --release` and `flutter build appbundle --release`.
   - iOS: `flutter build ios --no-codesign --release` and bundle unsigned IPA via payload compression.

## Verification
- Run `cd ui/miniai_app; flutter analyze --no-pub`.
- Run `cd ui/miniai_app; flutter test` ensuring all mobile interaction and gesture tests pass.

