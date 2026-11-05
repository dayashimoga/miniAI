# Stage & Release Certification Checklist

- [ ] All governance documents present and synchronized.
- [ ] `TODO.md` and `CHANGELOG.md` verified as strictly append-only.
- [ ] Rust `cargo fmt`, `cargo clippy`, and `cargo test` exit `0`.
- [ ] Python `pytest` exits `0` with `>= 90%` measured coverage on `miniai_ml`.
- [ ] Flutter `flutter analyze` and `flutter test --coverage` exit `0` with `>= 90%` coverage on core UI models/controllers.
- [ ] Zero secrets, mock data, or unconnected UI placeholders detected.
- [ ] Independent QA Auditor (`independent-qa-auditor`) verdict is `PASS`.

