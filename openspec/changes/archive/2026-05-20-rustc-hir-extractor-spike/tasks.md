## Phase 1: Implementation

- [x] I1 Add a pinned fixture and rustc HIR extractor spike helper that records toolchain, extractor, schema, context, observation hash, and caveats.
- [x] I2 Wire the spike into a bounded `object supplement` path while preserving Function Object v1 identity and pilot/spike naming.
- [x] I3 Add fail-closed replay comparison fields for rustc HIR spike receipts.
- [x] I4 Update contract metadata and docs to name fixture-bounded rustc HIR spike support without broad HIR/THIR/MIR overclaims.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Fixture HIR extraction attaches to Function Object v1" with focused cargo-octet fixture tests; save output at `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-fixture-attachment.txt`
- [x] V2 Verify scenario "HIR replay fails closed on context mismatch" with negative receipt fixtures; save output at `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-replay-negative.txt`
- [x] V3 Verify scenario "HIR spike support remains fixture bounded" with contract metadata and docs checks; save output at `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-fixture-bounded.txt`
- [x] V4 Verify scenario "HIR evidence has no correctness overclaim" with docs/output boundary checks; save output at `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-no-overclaim.txt`
- [x] V5 Verify aggregate Valence object evidence after the spike with `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; save output at `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/valence-object-evidence.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Fixture HIR extraction attaches to Function Object v1` -> `V1` -> `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-fixture-attachment.txt`
- `Scenario: HIR replay fails closed on context mismatch` -> `V2` -> `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-replay-negative.txt`
- `Scenario: HIR spike support remains fixture bounded` -> `V3` -> `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-fixture-bounded.txt`
- `Scenario: HIR evidence has no correctness overclaim` -> `V4` -> `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/hir-no-overclaim.txt`
- `Aggregate Valence object evidence gate` -> `V5` -> `openspec/changes/archive/2026-05-20-rustc-hir-extractor-spike/evidence/valence-object-evidence.txt`
