## Phase 1: Implementation

- [x] I1 Add capability vocabulary guidance standards documentation and policy classification
- [x] I2 Add implementation hooks, command contracts, or templates required by the capability vocabulary guidance design

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Capability name exposes authority" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/capability-name-exposes-authority.txt`
- [x] V2 Verify scenario "Core admissibility is explicit" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/core-admissibility-is-explicit.txt`
- [x] V3 Verify scenario "Test handler avoids ambient state" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/test-handler-avoids-ambient-state.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-capability-vocabulary-guidance proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Capability name exposes authority` -> `V1` -> `openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/capability-name-exposes-authority.txt`
- `Scenario: Core admissibility is explicit` -> `V2` -> `openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/core-admissibility-is-explicit.txt`
- `Scenario: Test handler avoids ambient state` -> `V3` -> `openspec/changes/archive/2026-05-05-add-capability-vocabulary-guidance/evidence/test-handler-avoids-ambient-state.txt`
