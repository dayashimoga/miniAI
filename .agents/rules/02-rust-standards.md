# Rule 02: Rust Core Engineering Standards

## Scope
Applies to `Cargo.toml` and all crates under `crates/`.

## Mandates
1. **Edition & Toolchain**:
   - Use Rust 2021 edition compatible with stable `rustc` and `cargo`.
2. **Error Handling & Safety**:
   - Define strongly typed domain errors using `thiserror` or explicit enums implementing `std::error::Error`.
   - Never use `.unwrap()` or `.expect()` in production library paths (`src/`); handle all errors explicitly with `Result<T, MiniAiError>`.
   - Forbid `unsafe` code (`#![forbid(unsafe_code)]`) in core business logic crates unless an explicit hardware FFI adapter crate requires it and documents safety invariants.
3. **Modularity & Runtime Independence**:
   - Keep `miniai-core` free of heavy model runtime dependencies.
   - Model runtimes (LLM, Vision, Image, Audio, Music, Video) must implement the `ModalityAdapter` trait independently and never force unrelated architectures into a single runtime.
4. **Verification**:
   - Every commit touching Rust code must pass `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.

