## Phase 1: Fresh-target rail

- [x] [serial] I1 Add a fresh-target object corpus receipt helper and deterministic integration test. ✅ evidence: `evidence/fresh-target-receipts.txt`
- [x] [serial] I2 Assert stable receipt bytes plus named compared fields and non-claim boundaries. ✅ evidence: `evidence/fresh-target-receipts.txt`

## Phase 2: Evidence and archive

- [x] [depends:I1,I2] V1 Capture focused fresh-target test evidence under `evidence/fresh-target-receipts.txt`. ✅ evidence: `evidence/fresh-target-receipts.txt`
- [x] [depends:V1] V2 Run full verification: `cargo fmt --check && cargo test -p cargo-octet && openspec validate --all --strict --json && git diff --check`. ✅ evidence: `evidence/final-verification.txt`
- [x] [depends:V2] H1 Archive this change after validation. command=`openspec archive fresh-object-corpus-determinism --yes`; input=`completed tasks and evidence`; asserts=`archive exists and no active change remains`; evidence=`openspec/changes/archive/<date>-fresh-object-corpus-determinism/evidence/final-verification.txt`

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Repeated object corpus receipts remain byte-identical across fresh target dirs | `evidence/fresh-target-receipts.txt` |
| Compared receipt fields remain stable across fresh target dirs | `evidence/fresh-target-receipts.txt` |
| Fresh-target receipt evidence preserves no-correctness and no-semantic-equivalence boundaries | `evidence/fresh-target-receipts.txt` |
