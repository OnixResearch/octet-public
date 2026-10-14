## Phase 1: Implementation

- [x] I1 Add remote effect contracts standards documentation and policy classification
- [x] I2 Add implementation hooks, command contracts, or templates required by the remote effect contracts design

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Remote task carries bounds" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/remote-task-carries-bounds.txt`
- [x] V2 Verify scenario "Receipt semantics are explicit" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/receipt-semantics-are-explicit.txt`
- [x] V3 Verify scenario "Weak static signal remains human reviewed" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/weak-static-signal-remains-human-reviewed.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-remote-effect-contracts proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/archive/2026-05-05-add-remote-effect-contracts/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Remote task carries bounds` -> `V1` -> `openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/remote-task-carries-bounds.txt`
- `Scenario: Receipt semantics are explicit` -> `V2` -> `openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/receipt-semantics-are-explicit.txt`
- `Scenario: Weak static signal remains human reviewed` -> `V3` -> `openspec/changes/archive/2026-05-05-add-remote-effect-contracts/evidence/weak-static-signal-remains-human-reviewed.txt`
