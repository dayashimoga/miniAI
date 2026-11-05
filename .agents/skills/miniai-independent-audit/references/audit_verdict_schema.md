# Independent Stage Gate Audit Verdict Schema

Every stage gate audit conducted by `independent-qa-auditor` and `arch-security-reviewer` must report:

```json
{
  "stage": "Stage 0",
  "auditor": "independent-qa-auditor",
  "verdict": "PASS",
  "checks": {
    "governance_files_complete": true,
    "append_only_logs_valid": true,
    "antigravity_skills_valid": true,
    "rust_tests_passed": true,
    "python_tests_passed": true,
    "flutter_tests_passed": true,
    "critical_coverage_above_90_percent": true,
    "zero_mock_or_placeholder_violations": true,
    "zero_secret_leaks": true
  }
}
```

