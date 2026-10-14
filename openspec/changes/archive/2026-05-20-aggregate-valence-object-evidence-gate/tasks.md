## Phase 1: Implementation

- [x] I1 Add explicit object-evidence subcheck derivations and aggregate `valence-object-evidence` flake check.
- [x] I2 Document the aggregate command and bounded non-proof scope in the contributor guide.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Aggregate gate composes object evidence rails" with the new Nix check; save output at `openspec/changes/archive/2026-05-20-aggregate-valence-object-evidence-gate/evidence/valence-object-evidence-check.txt`.
- [x] V2 Verify scenario "Contributor docs name the aggregate command and boundaries" with deterministic grep checks; save output at `openspec/changes/archive/2026-05-20-aggregate-valence-object-evidence-gate/evidence/docs-boundary-check.txt`.
- [x] V3 Verify the full Octet gate; save output at `openspec/changes/archive/2026-05-20-aggregate-valence-object-evidence-gate/evidence/final-verification.txt`.

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Aggregate gate composes object evidence rails` -> `V1` -> `openspec/changes/archive/2026-05-20-aggregate-valence-object-evidence-gate/evidence/valence-object-evidence-check.txt`
- `Scenario: Contributor docs name the aggregate command and boundaries` -> `V2` -> `openspec/changes/archive/2026-05-20-aggregate-valence-object-evidence-gate/evidence/docs-boundary-check.txt`
