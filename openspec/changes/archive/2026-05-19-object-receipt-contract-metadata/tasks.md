## Phase 1: Contract metadata

- [x] [serial] I1 Add Rust-derived object corpus contract metadata command for receipt/check-report surfaces. ✅ evidence: `evidence/contract-metadata.txt`
- [x] [serial] I2 Add positive and negative tests for metadata required fields, checked-in fixture compatibility, and boundary non-claims. ✅ evidence: `evidence/focused-tests.txt`

## Phase 2: Evidence and archive

- [x] [depends:I1,I2] V1 Capture contract metadata and focused test evidence under `openspec/changes/object-receipt-contract-metadata/evidence/`. ✅ evidence: `evidence/contract-metadata.txt`, `evidence/focused-tests.txt`
- [x] [depends:V1] V2 Run full verification: `cargo fmt --check && cargo test -p cargo-octet && openspec validate --all --strict --json && git diff --check`. ✅ evidence: `evidence/final-verification.txt`
- [x] [depends:V2] H1 Archive this change after validation. command=`openspec archive object-receipt-contract-metadata --yes`; input=`completed tasks and evidence`; asserts=`archive exists and no active change remains`; evidence=`openspec/changes/archive/<date>-object-receipt-contract-metadata/evidence/final-verification.txt`

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Contract metadata exposes receipt schema/version and required fields | `evidence/contract-metadata.txt` |
| Contract metadata exposes check-report schema/version and required fields | `evidence/contract-metadata.txt` |
| Checked-in internal corpus receipt fixture satisfies required fields and non-claim boundaries | `evidence/focused-tests.txt` |
| Negative fixture missing a required contract field fails closed | `evidence/focused-tests.txt` |
