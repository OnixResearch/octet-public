## Context

The archived `rustc-hir-extractor-spike` change introduced one pinned fixture and a rustc `-Z unpretty=hir-tree` observation supplement behind `--spike rustc-hir-fixture`. That fixture records exact rustc/toolchain/extractor/schema/context and fails closed on replay mismatches. The next useful step is not broad Rust coverage; it is one more bounded fixture shape with the same receipt rails.

## Goals

- Add one second checked-in fixture shape that differs structurally from the existing `checked(input + 1)` fixture.
- Keep fixture selection explicit, deterministic, and reviewable.
- Reuse the existing Function Object v1 attachment and fail-closed replay comparison model.
- Capture evidence proving both positive replay and negative shape/context mismatch behavior.

## Non-Goals

- Arbitrary user path extraction.
- Claiming rustc HIR observations are semantic proofs.
- Extending support status beyond fixture-backed/toolchain-scoped pilot evidence.

## Decisions

### 1. Add one named branch/control-flow fixture

**Choice:** Add a small second fixture function with a branch/control-flow shape, for example a threshold or clamp-style function.

**Rationale:** It exercises a different HIR observation shape while remaining small enough for deterministic evidence review.

**Rejected:** Adding a large fixture corpus. That would make review/evidence noisy and could imply broader HIR support than intended.

### 2. Keep fixture selection closed

**Choice:** Route the new shape through a closed selector or equivalent internal enum, not arbitrary path acceptance.

**Rationale:** Closed selection prevents operator-visible overclaims and keeps replay evidence bounded to checked-in fixture inputs.

### 3. Reuse fail-closed comparison fields

**Choice:** Compare the same identity/context fields as the first spike: schema version, source path, function name, rustc/toolchain identity, extractor identity, observation hash, caveats, and non-proof boundary fields.

**Rationale:** The value of the second shape is coverage breadth under the same safety contract, not a new receipt model.

## Risks / Trade-offs

- **Fixture selector naming drift**: Mitigate with tests/docs that call the shape fixture-backed and toolchain-scoped.
- **Stale object corpus fixtures**: If object/CLI changes affect corpus receipts, refresh fixtures with robust path handling as documented in `octet-workflow`.
- **Overclaim pressure**: Keep metadata/docs explicit that this is selected-fixture replay evidence only.
