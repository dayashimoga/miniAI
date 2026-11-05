# Mandatory Stage Gate Report Structure

At every stage gate, report:
1. **Changed Files**: Complete list of created/modified files.
2. **Architecture Decisions**: Summary of ADRs and design choices.
3. **Commands Executed**: Exact shell commands run during verification.
4. **Actual Test Results**: Real pass/fail counts and exit codes across Rust, Python, and Flutter.
5. **Measured Coverage**: Line/branch coverage percentages on critical business logic.
6. **Artifacts Produced**: Packages, reports, schemas, and stage prompts.
7. **Verification Classification & Limitations**: Breakdown of `PROVEN_ON_HOST`, `VERIFIED_IN_CI`, and `HARDWARE_DEPENDENT`.
8. **Remaining TODOs**: Next stage items logged in `TODO.md`.

