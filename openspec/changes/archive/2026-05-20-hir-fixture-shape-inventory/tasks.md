## Phase 1: Implementation

- [x] I1 Add read-only HIR fixture inventory CLI parsing and help surface.
- [x] I2 Generate fixture inventory data from the closed rustc HIR spike fixture registry.
- [x] I3 Update operator docs or contract text to reference inventory without broad HIR claims.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "HIR fixture inventory lists closed selectors" with focused tests and CLI smoke; save output at `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-list.txt`.
- [x] V2 Verify scenario "HIR fixture inventory preserves bounded support claims" with JSON/human assertions; save output at `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-boundaries.txt`.
- [x] V3 Verify scenario "HIR fixture inventory rejects unscoped requests" with negative parser/CLI smoke; save output at `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-negative.txt`.
- [x] V4 Run aggregate Valence object-evidence gate after inventory changes; save output at `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/valence-object-evidence.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review hir-fixture-shape-inventory CLI output and wording`; input=`inventory JSON/human output plus docs/contract wording`; asserts=`the inventory improves selector discoverability without implying general HIR support`; evidence=`openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-human-review.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: HIR fixture inventory lists closed selectors` -> `V1` -> `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-list.txt`
- `Scenario: HIR fixture inventory preserves bounded support claims` -> `V2` -> `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-boundaries.txt`
- `Scenario: HIR fixture inventory rejects unscoped requests` -> `V3` -> `openspec/changes/archive/2026-05-20-hir-fixture-shape-inventory/evidence/hir-fixture-inventory-negative.txt`
