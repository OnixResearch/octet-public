## Phase 1: Implementation

- [x] I1 Add one checked-in second rustc HIR fixture shape and closed fixture selector.
- [x] I2 Wire rustc HIR supplement emit/check to select the second fixture shape while preserving default first-fixture behavior.
- [x] I3 Update contract metadata and operator docs to name selected-fixture bounded support without broad HIR claims.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Second HIR fixture shape attaches to Function Object v1" with focused rustc HIR spike tests and CLI smoke; save output at `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-hir-fixture-attachment.txt`.
- [x] V2 Verify scenario "Second HIR fixture replay fails closed on shape mismatch" with focused negative replay test/smoke; save output at `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-hir-fixture-negative.txt`.
- [x] V3 Verify scenario "Multi-shape HIR spike support remains fixture bounded" with metadata/docs assertions; save output at `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-hir-fixture-bounded.txt`.
- [x] V4 Run aggregate Valence object-evidence gate after fixture-shape changes; save output at `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/valence-object-evidence.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review second-rustc-hir-fixture-shape fixture HIR observation and non-proof wording`; input=`cargo-octet/tests/fixtures/hir-spike/<second-fixture>.rs plus generated rustc HIR observation evidence`; asserts=`the added fixture shape broadens bounded observation evidence without implying general HIR support`; evidence=`openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-fixture-human-review.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Second HIR fixture shape attaches to Function Object v1` -> `V1` -> `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-hir-fixture-attachment.txt`
- `Scenario: Second HIR fixture replay fails closed on shape mismatch` -> `V2` -> `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-hir-fixture-negative.txt`
- `Scenario: Multi-shape HIR spike support remains fixture bounded` -> `V3` -> `openspec/changes/archive/2026-05-20-second-rustc-hir-fixture-shape/evidence/second-hir-fixture-bounded.txt`
