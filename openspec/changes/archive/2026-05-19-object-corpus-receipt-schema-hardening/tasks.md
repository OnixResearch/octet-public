## Phase 1: Implementation

- [x] I1 Add schema/version/source-selection validation for object corpus receipt checks.
- [x] I2 Extend object corpus check reports with typed invalid-input errors.
- [x] I3 Add regression tests for malformed receipts, unsupported schema/version, missing fields, invalid source selections, and empty corpora.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Malformed receipt emits typed invalid report" with focused cargo-octet tests; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-receipts.txt`
- [x] V2 Verify scenario "Unsupported schema is rejected before comparison" with focused cargo-octet tests; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-receipts.txt`
- [x] V3 Verify scenario "Missing receipt fields are rejected before comparison" with focused cargo-octet tests; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-receipts.txt`
- [x] V4 Verify scenario "Invalid source selection is rejected deterministically" with focused cargo-octet tests; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-source-selection.txt`
- [x] V5 Verify unchanged scenarios "Saved receipt replays cleanly", "Source drift fails the receipt check", and "Boundary tampering fails the receipt check" with existing corpus check tests; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/existing-check-contract.txt`
- [x] V6 Verify full repo slice with fmt, cargo-octet tests, OpenSpec validation, and whitespace checks; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/final-verification.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cargo octet object corpus check --receipt bad.json --output-format json lib.rs`; input=`malformed/future/missing-field receipt fixtures in cargo-octet integration tests`; asserts=`invalid reports are typed evidence-identity failures, not correctness or semantic-equivalence proofs`; evidence=`openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/schema-hardening-boundary.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Saved receipt replays cleanly` -> `V5` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/existing-check-contract.txt`
- `Scenario: Source drift fails the receipt check` -> `V5` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/existing-check-contract.txt`
- `Scenario: Boundary tampering fails the receipt check` -> `V5` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/existing-check-contract.txt`
- `Scenario: Malformed receipt emits typed invalid report` -> `V1` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-receipts.txt`
- `Scenario: Unsupported schema is rejected before comparison` -> `V2` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-receipts.txt`
- `Scenario: Missing receipt fields are rejected before comparison` -> `V3` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-receipts.txt`
- `Scenario: Invalid source selection is rejected deterministically` -> `V4` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-schema-hardening/evidence/invalid-source-selection.txt`
