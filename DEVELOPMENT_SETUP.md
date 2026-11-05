# MiniAI — Development Setup & Container Guide (`DEVELOPMENT_SETUP.md`)

**Document ID:** `MINIAI-DEV-001`  
**Version:** `1.0.0`

---

## 1. Prerequisites & Toolchain Matrix

MiniAI is designed to minimize host pollution by using workspace-local configurations and containerized environments wherever possible.

| Component | Toolchain | Tested Version | Purpose |
| :--- | :--- | :--- | :--- |
| **Version Control** | `git` | `>= 2.40` (`2.56.0` on host) | Source control & append-only log auditing |
| **Rust Core** | `rustc` / `cargo` | `>= 1.80` (`1.99.1` on host) | Core crates (`crates/miniai-core`) |
| **Flutter / Dart** | `flutter` / `dart` | `>= 3.24` (`3.38.4` / Dart `3.10.3` on host) | Cross-platform UI (`ui/miniai_app`) |
| **Python / ML** | `uv` | `>= 0.5` (`0.9.5` on host, Python `3.12+`) | Isolated Python/ML tooling (`ml/`) |
| **Containers** | Docker / DevContainer | Optional on host; standard in CI | Reproducible Linux build & test environment |

---

## 2. Quick Bootstrap & Verification (Host)

Run the following commands from the repository root (`g:\miniAI`):

### 2.1 Verify Rust Workspace
```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### 2.2 Verify Python ML & Governance Suite (via `uv`)
```powershell
Push-Location ml
uv run pytest --cov=miniai_ml --cov-report=term-missing --cov-fail-under=90
Pop-Location
```

### 2.3 Verify Flutter Cross-Platform UI
```powershell
Push-Location ui/miniai_app
flutter pub get
flutter analyze --no-pub
flutter test --coverage
Pop-Location
```

### 2.4 Run Master Stage 0 Gate Verifier
```powershell
uv run --project ml python scripts/verify_stage0.py
```

---

## 3. Containerized Development (`Docker` & `DevContainers`)

To run builds and tests inside an isolated Linux container without installing toolchains on the host:

```bash
# Build and run the verification suite via Docker Compose
docker compose run --rm miniai-dev
```

Or open the repository in VS Code with the **Dev Containers** extension, which automatically reads `.devcontainer/devcontainer.json`.

