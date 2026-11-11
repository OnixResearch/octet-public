## Phase 1: Contract Package

- [x] I1 Define the Octet-native file-format proposal/design/spec package for content-addressed Rust evidence.
- [x] V1 Validate the Octet file-format contract change and all OpenSpec content. evidence=openspec/changes/archive/2026-05-19-octet-evidence-format-contracts/evidence/final-verification.txt

## Verification Coverage

| Scenario | Verification |
| --- | --- |
| Markdown remains human review surface | V1 |
| JSON remains current machine receipt surface | V1 |
| Rust DTO owns emitted receipt shape | V1 |
| Nickel metadata tracks compatibility policy | V1 |
| JSON hash material is canonical and bounded | V1 |
| Future binary format is additive | V1 |
| File format does not prove correctness | V1 |
| File format does not prove semantic equivalence | V1 |
