# GitHub Actions Target & Runner Matrix

| Target Platform | GitHub Actions Runner | Build / Test Artifacts |
| :--- | :--- | :--- |
| Windows (`x86_64`) | `windows-latest` | Rust `cargo test`, Python `pytest`, Flutter `build windows` |
| Linux (`x86_64`) | `ubuntu-latest` | Rust `cargo test`, Python `pytest`, Flutter `build linux` |
| macOS (`arm64`/`x86_64`) | `macos-latest` | Rust `cargo test`, Python `pytest`, Flutter `build macos` |
| Android (`arm64-v8a`) | `ubuntu-latest` (Java 17 + Android SDK) | Flutter `build apk --debug` / `--release` |
| iOS (`arm64`) | `macos-latest` (Xcode) | Flutter `build ios --release --no-codesign` |

