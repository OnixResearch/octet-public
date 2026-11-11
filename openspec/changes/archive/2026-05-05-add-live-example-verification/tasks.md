## Phase 1: Implementation

- [x] I1 Add live example verification standards documentation and policy classification
- [x] I2 Add implementation hooks, command contracts, or templates required by the live example verification design

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Example links to fixture" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/example-links-to-fixture.txt`
- [x] V2 Verify scenario "Prose-only example is explicit" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/prose-only-example-is-explicit.txt`
- [x] V3 Verify scenario "Changed fixture invalidates manifest" with deterministic docs/policy/fixture checks; save output at `openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/changed-fixture-invalidates-manifest.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review add-live-example-verification proposal/design/specs against Unison-inspired source concept and Tigerstyle FCIS constraints`; input=`openspec/changes/archive/2026-05-05-add-live-example-verification/proposal.md, design.md, specs/documentation/spec.md`; asserts=`scope imports the useful discipline without copying Unison syntax or premature hard gates`; evidence=`openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Example links to fixture` -> `V1` -> `openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/example-links-to-fixture.txt`
- `Scenario: Prose-only example is explicit` -> `V2` -> `openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/prose-only-example-is-explicit.txt`
- `Scenario: Changed fixture invalidates manifest` -> `V3` -> `openspec/changes/archive/2026-05-05-add-live-example-verification/evidence/changed-fixture-invalidates-manifest.txt`
