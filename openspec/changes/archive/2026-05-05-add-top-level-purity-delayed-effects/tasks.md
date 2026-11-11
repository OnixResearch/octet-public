## Phase 1: Implementation

- [x] I1 Add top level purity and delayed effects standards documentation and policy classification
- [x] I2 Add implementation hooks, command contracts, or templates required by the top level purity and delayed effects design

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Initializer side effects are rejected by guidance" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/initializer-side-effects-are-rejected-by-guidance.txt`
- [x] V2 Verify scenario "Constructors validate only" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/constructors-validate-only.txt`
- [x] V3 Verify scenario "Effect is represented as data" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/effect-is-represented-as-data.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-top-level-purity-delayed-effects proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/proposal.md, design.md, specs/standards-codification/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Initializer side effects are rejected by guidance` -> `V1` -> `openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/initializer-side-effects-are-rejected-by-guidance.txt`
- `Scenario: Constructors validate only` -> `V2` -> `openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/constructors-validate-only.txt`
- `Scenario: Effect is represented as data` -> `V3` -> `openspec/changes/archive/2026-05-05-add-top-level-purity-delayed-effects/evidence/effect-is-represented-as-data.txt`
