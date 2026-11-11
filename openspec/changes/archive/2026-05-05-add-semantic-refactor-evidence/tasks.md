## Phase 1: Implementation

- [x] I1 Add semantic refactor evidence standards documentation and policy classification
- [x] I2 Add implementation hooks, command contracts, or templates required by the semantic refactor evidence design

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Move record preserves identity" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/move-record-preserves-identity.txt`
- [x] V2 Verify scenario "Rename record states rationale" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/rename-record-states-rationale.txt`
- [x] V3 Verify scenario "Machine output is stable" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/machine-output-is-stable.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-semantic-refactor-evidence proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/proposal.md, design.md, specs/consumer-integration/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Move record preserves identity` -> `V1` -> `openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/move-record-preserves-identity.txt`
- `Scenario: Rename record states rationale` -> `V2` -> `openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/rename-record-states-rationale.txt`
- `Scenario: Machine output is stable` -> `V3` -> `openspec/changes/archive/2026-05-05-add-semantic-refactor-evidence/evidence/machine-output-is-stable.txt`
