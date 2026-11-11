## Phase 1: Implementation

- [x] I1 Add the typed config/receipt family registry and documentation entries for current Tiger Style config, generated policy artifact, proof receipt, closure receipt, and excluded non-candidate families.
- [x] I2 Add Nickel-authored config contracts and deterministic exports for the first human-maintained run/profile/fixture config family.
- [x] I3 Add Rust-derived Nickel contract generation and freshness checks for the first receipt DTO family.
- [x] I4 Add structured acceptance receipt fields that bind emitted evidence to config digest or exported config artifact identity.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "New family declares one source of truth" with the registry checker; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/registry-source-of-truth.txt`
- [x] V2 Verify scenario "Non-candidates are explicit" with registry negative fixtures; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/registry-non-candidates.txt`
- [x] V3 Verify scenario "Human config exports deterministic tool input" with Nickel export assertions; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/human-config-export.txt`
- [x] V4 Verify scenario "Invalid human config fails before runtime side effects" with Nickel negative fixtures; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/human-config-negative.txt`
- [x] V5 Verify scenario "Receipt contract freshness detects drift" with the generated-contract freshness checker; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-contract-freshness.txt`
- [x] V6 Verify scenario "Receipt fixtures validate shape without owning behavior" with positive and negative receipt fixtures; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-fixtures.txt`
- [x] V7 Verify scenario "Receipt binds emitted evidence to config digest" with a focused receipt-emission test; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-config-digest.txt`
- [x] V8 Verify scenario "Raw logs remain debug aids" with receipt/log acceptance checks; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-log-boundary.txt`
- [x] V9 Verify focused Nickel/OpenSpec/Rust/whitespace checks; save output at `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review registry classifications and generated-contract ownership`; input=`config/receipt registry diff and generated contract diff`; asserts=`human-authored config is Nickel-owned, runtime receipts are Rust-derived, and non-candidates do not expose secrets or runtime internals to Nickel`; evidence=`openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/ownership-review.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: New family declares one source of truth` -> `V1` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/registry-source-of-truth.txt`
- `Scenario: Non-candidates are explicit` -> `V2` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/registry-non-candidates.txt`
- `Scenario: Human config exports deterministic tool input` -> `V3` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/human-config-export.txt`
- `Scenario: Invalid human config fails before runtime side effects` -> `V4` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/human-config-negative.txt`
- `Scenario: Receipt contract freshness detects drift` -> `V5` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-contract-freshness.txt`
- `Scenario: Receipt fixtures validate shape without owning behavior` -> `V6` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-fixtures.txt`
- `Scenario: Receipt binds emitted evidence to config digest` -> `V7` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-config-digest.txt`
- `Scenario: Raw logs remain debug aids` -> `V8` -> `openspec/changes/archive/2026-05-06-add-nickel-config-receipt-boundaries/evidence/receipt-log-boundary.txt`
