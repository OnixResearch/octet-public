## Phase 1: Implementation

- [x] I1 Add static negative receipt fixtures for malformed JSON, duplicate receipt source paths, boundary tamper, and empty receipt source paths.
- [x] I2 Add a non-Rust source-selection fixture and extend compatibility tests for all new fail-closed cases.

## Phase 2: Deterministic verification

- [x] V1 Verify scenarios "Boundary tamper fixture fails closed", "Duplicate receipt source path fixture fails closed", "Malformed receipt fixture fails closed", "Empty receipt source path fixture fails closed", and "Unsupported non-Rust source selection fixture fails closed" with focused compatibility tests; save output at `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/compatibility-tests.txt`.
- [x] V2 Verify full Octet gate; save output at `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/final-verification.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`python3 - <<'PY' ... verify checked-in negative fixture inventory ... PY`; input=`cargo-octet/tests/fixtures/object-corpus/compatibility`; asserts=`negative fixture inventory includes malformed JSON, duplicate source path, boundary tamper, empty receipt source path, and non-Rust source-selection fixture`; evidence=`openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/fixture-inventory.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Boundary tamper fixture fails closed` -> `V1` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/compatibility-tests.txt`
- `Scenario: Duplicate receipt source path fixture fails closed` -> `V1` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/compatibility-tests.txt`
- `Scenario: Malformed receipt fixture fails closed` -> `V1` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/compatibility-tests.txt`
- `Scenario: Empty receipt source path fixture fails closed` -> `V1` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/compatibility-tests.txt`
- `Scenario: Unsupported non-Rust source selection fixture fails closed` -> `V1` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-compatibility-negatives/evidence/compatibility-tests.txt`
