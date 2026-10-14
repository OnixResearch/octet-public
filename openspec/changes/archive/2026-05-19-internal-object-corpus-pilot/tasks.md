## Phase 1: Implementation

- [x] I1 Add `object corpus receipt` CLI parsing and help text.
- [x] I2 Implement deterministic object corpus receipt generation.
- [x] I3 Expand the internal Octet object corpus test set.
- [x] I4 Add OpenSpec delta/spec/design/tasks for the pilot.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Expanded internal corpus emits a deterministic receipt" with targeted cargo-octet test; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-pilot/evidence/internal-corpus-receipt.txt`
- [x] V2 Verify scenario "Receipt preserves no-correctness boundary" with targeted cargo-octet test; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-pilot/evidence/internal-corpus-boundary.txt`
- [x] V3 Verify change validation with OpenSpec strict validation; save output at `openspec/changes/archive/2026-05-19-internal-object-corpus-pilot/evidence/openspec-validate.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cargo run -p cargo-octet --bin cargo-octet -- object corpus receipt --output /tmp/internal-corpus.json cargo-octet/src/*.rs`; input=`Octet main checkout source files`; asserts=`receipt is an internal pilot receipt and not downstream CAF support`; evidence=`openspec/changes/archive/2026-05-19-internal-object-corpus-pilot/evidence/oracle-boundary.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Expanded internal corpus emits a deterministic receipt` -> `V1` -> `openspec/changes/archive/2026-05-19-internal-object-corpus-pilot/evidence/internal-corpus-receipt.txt`
- `Scenario: Receipt preserves no-correctness boundary` -> `V2` -> `openspec/changes/archive/2026-05-19-internal-object-corpus-pilot/evidence/internal-corpus-boundary.txt`
