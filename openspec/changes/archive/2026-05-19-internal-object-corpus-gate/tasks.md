## Phase 1: Implementation

- [x] I1 Add a checked-in internal object corpus receipt fixture under cargo-octet integration fixtures
- [x] I2 Add a corpus gate integration test that replays the checked-in fixture

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Checked-in internal corpus receipt replays cleanly" with focused gate test; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/corpus-gate-pass.txt`
- [x] V2 Verify scenario "Internal corpus gate preserves evidence-only boundaries" with focused gate test; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/corpus-gate-boundary.txt`
- [x] V3 Verify scenario "Fixture refresh is deterministic" with fixture refresh comparison; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/fixture-refresh.txt`
- [x] V4 Verify full Octet gate; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/final-verification.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`nix develop -c bash -c 'cargo run -p cargo-octet --bin cargo-octet -- object corpus receipt --output /tmp/internal-corpus-receipt.json cargo-octet/src/*.rs && cmp /tmp/internal-corpus-receipt.json cargo-octet/tests/fixtures/object-corpus/internal-corpus-receipt.json'`; input=`cargo-octet/src/*.rs and checked-in fixture`; asserts=`checked-in receipt fixture was generated from the bounded internal corpus`; evidence=`openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/fixture-refresh.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Checked-in internal corpus receipt replays cleanly` -> `V1` -> `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/corpus-gate-pass.txt`
- `Scenario: Internal corpus gate preserves evidence-only boundaries` -> `V2` -> `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/corpus-gate-boundary.txt`
- `Scenario: Fixture refresh is deterministic` -> `V3` -> `openspec/changes/archive/2026-05-19-internal-object-corpus-gate/evidence/fixture-refresh.txt`
