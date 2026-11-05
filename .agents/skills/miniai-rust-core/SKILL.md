---
name: miniai-rust-core
description: >-
  Guides development, linting, error handling, and testing of MiniAI Rust crates under crates/.
  Activate when implementing Rust services, domain models, hardware probes, storage layers,
  or runtime adapter traits.
---

# MiniAI Rust Core Engineering Skill

Use this skill when creating or modifying any crate in the Rust workspace (`Cargo.toml`, `crates/*`).

## Standards Checklist
1. **Safety**: Keep `#![forbid(unsafe_code)]` in `miniai-core` and business logic crates.
2. **Error Handling**: Use `MiniAiError` (`thiserror::Error`) for all fallible operations. Never call `.unwrap()` or `.expect()` in library code (`src/`).
3. **Serialization**: Derive `serde::Serialize` and `serde::Deserialize` with explicit `#[serde(rename_all = "snake_case")]` where external contracts apply.
4. **Testing**: Write unit tests inside each module (`#[cfg(test)] mod tests`) and integration/contract tests in `crates/<crate>/tests/`.

## Workflow Steps
1. Review [`examples/adapter_trait_example.rs`](./examples/adapter_trait_example.rs) for idiomatic trait and error design.
2. Implement the minimal required structs, traits, and methods.
3. Run formatting check:
   `cargo fmt --all -- --check`
4. Run strict Clippy lints:
   `cargo clippy --workspace --all-targets -- -D warnings`
5. Execute unit and contract test suites:
   `cargo test --workspace`

## Verification
- All three commands (`fmt`, `clippy`, `test`) must exit with code `0`.

