## Phase 1: Implementation

- [x] I1 Add structural and unique type guidance standards documentation and policy classification
- [x] I2 Add implementation hooks, command contracts, or templates required by the structural and unique type guidance design

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Identity values use nominal wrappers" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/identity-values-use-nominal-wrappers.txt`
- [x] V2 Verify scenario "Structural records remain intentional" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/structural-records-remain-intentional.txt`
- [x] V3 Verify scenario "Primitive lint needs evidence" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/primitive-lint-needs-evidence.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-structural-unique-type-guidance proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Identity values use nominal wrappers` -> `V1` -> `openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/identity-values-use-nominal-wrappers.txt`
- `Scenario: Structural records remain intentional` -> `V2` -> `openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/structural-records-remain-intentional.txt`
- `Scenario: Primitive lint needs evidence` -> `V3` -> `openspec/changes/archive/2026-05-05-add-structural-unique-type-guidance/evidence/primitive-lint-needs-evidence.txt`
