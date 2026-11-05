# Test & Coverage Execution Commands

```powershell
# 1. Rust Core Verification
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 2. Python ML & Governance Verification (>=90% coverage enforced)
Push-Location ml
uv run pytest --cov=miniai_ml --cov-report=term-missing --cov-fail-under=90
Pop-Location

# 3. Flutter UI Verification
Push-Location ui/miniai_app
flutter analyze --no-pub
flutter test --coverage
Pop-Location
```

