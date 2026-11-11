## Phase 1: Implementation

- [x] I1 Add CLI parsing/help for `object corpus check --receipt RECEIPT.json [PATHS...]`.
- [x] I2 Implement receipt replay comparison and deterministic check reports.
- [x] I3 Add corpus check integration tests for pass, source drift, and boundary tamper cases.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Saved corpus receipt replays cleanly" with focused cargo test; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/corpus-check-pass.txt`
- [x] V2 Verify scenario "Source drift fails corpus receipt check" with focused cargo test; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/corpus-check-drift.txt`
- [x] V3 Verify scenario "Boundary tamper fails corpus receipt check" with focused cargo test; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/corpus-check-boundary.txt`
- [x] V4 Verify all cargo-octet tests and OpenSpec strict validation; save output at `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/final-verification.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Saved corpus receipt replays cleanly` -> `V1` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/corpus-check-pass.txt`
- `Scenario: Source drift fails corpus receipt check` -> `V2` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/corpus-check-drift.txt`
- `Scenario: Boundary tamper fails corpus receipt check` -> `V3` -> `openspec/changes/archive/2026-05-19-object-corpus-receipt-check/evidence/corpus-check-boundary.txt`
