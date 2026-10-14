## Phase 1: Implementation

- [x] I1 Extend Nickel and Rust policy models with generated type-state mechanism obligation fields
- [x] I2 Export type-state obligations through generated policy artifacts and checker config
- [x] I3 Implement artifact-backed type-state structural diagnostics in `tigerstyle-standards`
- [x] I4 Add compliant and noncompliant type-state protocol fixtures
- [x] I5 Update docs describing type-state structural mechanism artifacts

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Type-state obligations export deterministically" with artifact export assertions; save output at `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/policy-artifact-export.txt`
- [x] V2 Verify scenario "Compliant nominal marker-state API passes" with standards fixture check; save output at `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- [x] V3 Verify scenario "Raw protocol bypass fails structurally" with standards fixture check; save output at `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- [x] V4 Verify scenario "Focused Tigerstyle checks pass" with Rust/Nix/OpenSpec commands; save output at `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review design proof plan before archive`; input=`openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/design.md and evidence transcripts`; asserts=`the change claims structural API enforcement only, not verified protocol correctness`; evidence=`openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/proof-plan-review.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Type-state obligations export deterministically` -> `V1` -> `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/policy-artifact-export.txt`
- `Scenario: Compliant nominal marker-state API passes` -> `V2` -> `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- `Scenario: Raw protocol bypass fails structurally` -> `V3` -> `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/type-state-fixtures.txt`
- `Scenario: Focused Tigerstyle checks pass` -> `V4` -> `openspec/changes/archive/2026-05-05-enforce-type-state-protocol-structural-gate/evidence/focused-checks.txt`
