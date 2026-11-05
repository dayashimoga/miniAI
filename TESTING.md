# MiniAI — Testing, Quality Assurance & Coverage Strategy (`TESTING.md`)

**Document ID:** `MINIAI-TEST-001`  
**Version:** `1.0.0`

---

## 1. Testing Philosophy

In MiniAI, **no feature is complete until it is implemented, integrated, executed, tested, documented, and independently verified**. Tests must never rely on fabricated results or simulated mocks in production code paths.

---

## 2. Multi-Tier Test Architecture

| Tier | Scope | Location | Execution Command |
| :--- | :--- | :--- | :--- |
| **1. Unit Tests** | Pure domain logic, error branches, hardware selection, privacy policy | `crates/miniai-core/src/lib.rs`, `ml/tests/`, `ui/miniai_app/test/` | `cargo test`, `uv run pytest`, `flutter test` |
| **2. Contract Tests** | Cross-language JSON schema parity (`Rust` <-> `Dart` <-> `Python`) | `crates/miniai-core/tests/contract_tests.rs`, `ml/tests/test_foundation.py` | `cargo test --test contract_tests` |
| **3. UI & Accessibility Tests** | Widget rendering, `Semantics` labels, privacy toggles, responsive layout | `ui/miniai_app/test/widget_test.dart` | `flutter test --coverage` |
| **4. Offline & Recovery Tests** | `LocalOnlyPrivacyMode` enforcement, CPU fallback on missing GPU, corrupt SHA-256 rejection | All 3 test suites | Included in workspace test runs |
| **5. Security Tests** | Path traversal (`..`) blocking, expired consent token rejection, secret scanning | `crates/miniai-core`, `scripts/verify_stage0.py` | `uv run --project ml python scripts/verify_stage0.py` |
| **6. Governance & Log Integrity** | Append-only `TODO.md`/`CHANGELOG.md` verification, Antigravity skill/rule validation | `ml/src/miniai_ml/governance.py` | `uv run --project ml python scripts/verify_stage0.py` |

---

## 3. Coverage Requirements & Measurement

- **Target Threshold**: Minimum **90% measured coverage** on critical business logic (`crates/miniai-core`, `ml/src/miniai_ml`, and `ui/miniai_app/lib/src/models` + `controllers`).
- **Required Pass Rate**: **100%** of active tests must pass at every Stage Gate.

### Truthful Result Classification
Every test execution and benchmark report classifies results into one of three honest categories:
1. `PROVEN_ON_HOST`: Directly executed and verified on the local workstation (`Windows x86_64`, CPU execution path).
2. `VERIFIED_IN_CI`: Configured and executed on GitHub Actions target OS runners (`ubuntu-latest`, `macos-latest`, `windows-latest`).
3. `HARDWARE_DEPENDENT`: Requires specific accelerator hardware (e.g., NVIDIA CUDA GPU or Apple Silicon Metal) not physically attached to the current machine; CPU fallback path is 100% verified.

