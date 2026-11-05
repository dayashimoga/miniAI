# MiniAI — Codebase Structure & Developer Navigation Guide (`CODEBASE_GUIDE.md`)

**Document ID:** `MINIAI-GUIDE-001`  
**Version:** `1.0.0`

---

## 1. Repository Layout

```text
g:\miniAI\
├── .agents/                          # Antigravity Workspace Customizations
│   ├── rules/                        # Always-on modular engineering rules (01-06)
│   └── skills/                       # 18 reusable procedural skills + SKILLS_INDEX.md
├── .devcontainer/                    # VS Code / DevContainer reproducible environment
│   └── devcontainer.json
├── .github/                          # GitHub Actions Multi-Platform CI/CD
│   └── workflows/
│       ├── ci.yml                    # Build & test across Windows, Linux, macOS, Android, iOS
│       ├── security.yml              # Secret, dependency & governance security audits
│       └── release.yml               # Multi-OS artifact packaging & SHA-256 checksums
├── crates/                           # Rust Core Workspace
│   └── miniai-core/                  # Core contracts, privacy engine, hardware probe, runtime registry
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── docs/                             # Engineering Specifications & Stage Prompts
│   └── stage_prompts/                # Executable technical prompts for Stages 00 through 10
│       ├── STAGE_00_PROMPT.md
│       └── ... (STAGE_01 through STAGE_10)
├── ml/                               # Python / PyTorch ML & Governance Subsystem (managed by uv)
│   ├── pyproject.toml
│   ├── src/
│   │   └── miniai_ml/
│   │       ├── __init__.py
│   │       ├── contracts.py          # Cross-language schema & SHA-256/path validators
│   │       ├── governance.py         # Append-only log, skill/rule & repository auditor
│   │       └── runtime_probe.py      # CPU-first ML hardware & deterministic seed manager
│   └── tests/
│       └── test_foundation.py        # Unit, contract & security tests (>=90% coverage)
├── scripts/                          # Automation & Stage Gate Verification Scripts
│   └── verify_stage0.py              # Master Stage 0 deterministic gate verifier
├── ui/                               # Cross-Platform Flutter UI
│   └── miniai_app/
│       ├── pubspec.yaml
│       ├── analysis_options.yaml
│       ├── lib/
│       │   ├── main.dart
│       │   └── src/
│       │       ├── models/
│       │       │   └── system_contracts.dart
│       │       ├── controllers/
│       │       │   └── platform_controller.dart
│       │       └── screens/
│       │           └── dashboard_screen.dart
│       └── test/
│           └── widget_test.dart
├── Cargo.toml                        # Root Rust Workspace Manifest
├── Dockerfile                        # Multi-stage non-root container image
├── docker-compose.yml                # Local reproducible container orchestration
└── *.md                              # Root Governance, Architecture & Append-Only Log Documents
```

---

## 2. Subsystem Responsibilities & Ownership

### 2.1 Rust Core (`crates/miniai-core`)
- **`Modality` & `AcceleratorBackend`**: Canonical enums representing all 7 independent AI modalities and supported compute backends (`Cpu`, `Cuda`, `Vulkan`, `Metal`, `DirectMl`, `Rocm`, `CoreMl`, `Nnapi`).
- **`HardwareDetector` & `HardwareProfile`**: Probes host CPU threads, system memory, and accelerators; computes 85% memory admission bounds and selects execution backends with deterministic CPU fallback.
- **`PrivacyPolicyEngine` & `CloudConsentToken`**: Enforces `LocalOnlyPrivacyMode` (default `true`) and validates scoped, time-bounded consent tokens before allowing any hybrid cloud execution.
- **`RuntimeRegistry` & `ModalityAdapter`**: Registers independent modality adapters and aggregates system health snapshots.
- **Security Utilities**: `verify_sha256_digest` and `validate_safe_relative_path`.

### 2.2 Flutter Cross-Platform UI (`ui/miniai_app`)
- **`system_contracts.dart`**: Strongly typed Dart representations mirroring `miniai-core` JSON contracts.
- **`platform_controller.dart`**: Reactive `ChangeNotifier` managing `LocalOnlyPrivacyMode`, explicit `CloudConsentToken` authorization/revocation, CPU-only override, and modality adapter health inspection.
- **`dashboard_screen.dart`**: Accessible, responsive multi-pane shell displaying live privacy state, hardware admission telemetry, and independent modality adapter cards.

### 2.3 Python ML & Governance (`ml/src/miniai_ml`)
- **`contracts.py`**: Python dataclasses mirroring Rust/Flutter contracts, SHA-256 file/bytes verification, and safe path validation.
- **`runtime_probe.py`**: Deterministic random seeding (`seed_everything`) and CPU-first ML runtime capability probe.
- **`governance.py`**: Automated auditor validating required governance docs, Antigravity rules, 18 skills (`SKILL.md` frontmatter + linked references), and strictly append-only `TODO.md` / `CHANGELOG.md` logs.

