## Implementation Tasks

- [x] [serial] I1 Add a named flake check for the internal object corpus gate. ✅ evidence: `evidence/object-corpus-gate-check.txt`
- [x] [serial] I2 Document the named check and its bounded evidence claim. ✅ evidence: `evidence/final-verification.txt`

## Verification Tasks

- [x] [depends:I1] V1 Run the named object corpus gate check and save output at `evidence/object-corpus-gate-check.txt`.
- [x] [depends:V1] V2 Run full verification: `cargo fmt --check && cargo test -p cargo-octet && openspec validate --all --strict --json && git diff --check`; save output at `evidence/final-verification.txt`.
- [x] [depends:V2] H1 Archive the change after validation and keep replay evidence under the archive path.

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Object corpus CI check covers fixture replay | `evidence/object-corpus-gate-check.txt` |
| Object corpus CI check covers contract metadata | `evidence/object-corpus-gate-check.txt` |
| Object corpus CI check covers fresh-target determinism | `evidence/object-corpus-gate-check.txt` |
| Object corpus CI check preserves bounded non-claims | `evidence/final-verification.txt` |
