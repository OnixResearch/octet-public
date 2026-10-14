# Tasks

## Implementation

- [x] [serial] I1 Add checked-in object corpus compatibility source and receipt fixtures.
- [x] [depends:I1] I2 Add tests that replay the positive v1 fixture and reject named negative fixtures with typed errors.

## Verification

- [x] [depends:I2] V1 Run focused object corpus compatibility tests and save output to `evidence/object-corpus-compatibility-tests.txt`.
- [x] [depends:V1] V2 Run full verification: `cargo fmt --check && cargo test -p cargo-octet && openspec validate --all --strict --json && git diff --check`; save output to `evidence/final-verification.txt`.

## Human / Review

- [x] [depends:V2] H1 Human/oracle checkpoint: command=`review object-corpus-compatibility-fixtures`; input=`fixtures, tests, final verification`; asserts=`fixtures document accepted v1 and rejected future/missing-field behavior without correctness or semantic-equivalence overclaim`; evidence=`evidence/final-verification.txt`.

## Change Status

- Status: complete
- Last updated: 2026-05-19

## Verification Coverage

| Scenario | Tasks | Evidence |
| --- | --- | --- |
| Positive v1 compatibility fixture replays cleanly | I1, I2, V1 | `evidence/object-corpus-compatibility-tests.txt` |
| Negative compatibility fixtures fail closed with typed errors | I1, I2, V1 | `evidence/object-corpus-compatibility-tests.txt` |
| No correctness or semantic-equivalence boundary overclaim | I2, V2, H1 | `evidence/final-verification.txt` |
