## Context

The selected HIR and THIR inventories already include `branch-control-flow`; MIR selected-fixture evidence currently includes `checked-arithmetic` and `call-return`. Recent object-corpus freshness work added a named rail that must be run whenever these inputs or inventories change.

## Goals / Non-Goals

**Goals:**
- Add a checked-in MIR `branch-control-flow` fixture and selector.
- Keep MIR evidence selected-fixture-only and operator-visible through the existing MIR inventory and aggregate Valence gates.
- Preserve explicit non-claim boundaries.

**Non-Goals:**
- General MIR extraction.
- Default/non-fixture MIR support.
- Any semantic equivalence or correctness proof.

## Decisions

### 1. Reuse the bounded fixture-selector pattern

**Choice:** Add `branch-control-flow` beside the existing MIR `checked-arithmetic` and `call-return` selectors.

**Rationale:** This follows the THIR branch fixture shape and keeps support closed over explicit checked-in fixtures.

**Alternative:** Promote broader MIR extraction. Rejected because the evidence ladder intentionally advances by selected fixture, not by broad claims.

### 2. Extend existing MIR inventory/aggregate rails

**Choice:** Update existing MIR inventory tests and `mir-supplement-inventory-gate` rather than adding a new aggregate family.

**Rationale:** The operator already has a named MIR inventory rail and aggregate Valence gate; this change extends their selected set while preserving boundaries.

### 3. Run fixture freshness after inventory changes

**Choice:** Regenerate and verify object-corpus fixture receipts/taxonomy if freshness indicates drift.

**Rationale:** CLI/object/test source changes affect the selected internal object corpus; the new freshness rail exists to prevent stale checked-in receipts.

## Risks / Mitigations

- **Overclaiming MIR support** → keep docs/spec/tests saying selected-fixture-only and default/non-fixture `unsupported-planned`.
- **Stale object-corpus fixtures** → run `object-corpus-fixture-freshness` and refresh fixtures before archive.
- **Inventory asymmetry regression** → assert THIR inventory unchanged and MIR inventory exactly includes `checked-arithmetic`, `call-return`, and `branch-control-flow`.

## Validation Plan

- Focused Rust tests: `mir_fixture`, `thir_inventory`, and object-corpus freshness tests.
- Nix rails: `mir-supplement-inventory-gate`, `object-corpus-fixture-freshness`, and `valence-object-evidence`.
- Boundary checker over docs/spec/evidence.
- `openspec validate --all --strict --json` and `git diff --check`.
