## Context

The MIR selected-fixture ladder now supports exact selectors `checked-arithmetic` and `call-return`. The existing focused test `mir_fixture` covers inventory, positive emit/check replay, unsupported selector rejection, and default `unsupported-planned` boundaries. The aggregate Nix gate currently reports MIR through `mir_supplement_fixture_gate`, while THIR has a more explicit inventory rail in the aggregate gate.

## Goals / Non-Goals

**Goals:**
- Add a focused MIR inventory Nix rail that is parallel to the THIR inventory rail.
- Prove inventory output and accepted emit/check commands stay derived from the same selected-fixture registry.
- Compose the rail into `valence-object-evidence` with a distinct PASS line.
- Preserve selected-fixture-only and non-proof boundaries in docs and output.

**Non-Goals:**
- Add new MIR fixture shapes.
- Change MIR default/non-fixture status from `unsupported-planned`.
- Replace the existing MIR fixture aggregate output line without an explicit compatibility decision.
- Claim general MIR extraction or semantic correctness.

## Decisions

### 1. Add a focused MIR supplement inventory Nix rail

**Choice:** Add a named check such as `checks.x86_64-linux.mir-supplement-inventory-gate` that runs the focused MIR inventory/roundtrip tests.

**Covers:** `Aggregate gate composes MIR inventory rail`; `MIR fixture inventory commands roundtrip selected fixture evidence`.

**Rationale:** A named rail makes the operator-visible evidence shape symmetrical with THIR and gives aggregate CI a stable dependency to compose.

**Alternative considered:** Keep only `mir-supplement-fixture-gate`. Rejected because the name does not communicate inventory/roundtrip coverage and is easier to confuse with a one-shape fixture smoke.

**Implementation:** Reuse the existing focused MIR test target if it already asserts inventory and roundtrip behavior; otherwise split or add a test name that specifically covers inventory-listed emit/check command replay.

**Boundary cases:** The rail must still reject aliases such as `checked` and non-selected shapes such as `branch-control-flow`.

### 2. Compose the rail into aggregate Valence evidence

**Choice:** Make `valence-object-evidence` depend on the MIR inventory rail and print `valence_object_evidence.mir_supplement_inventory_gate: PASS`.

**Covers:** `Aggregate gate composes object evidence rails`; `Aggregate gate composes MIR inventory rail`.

**Rationale:** The aggregate gate is the operator entry point. It should report the MIR rail with the same specificity as THIR.

**Alternative considered:** Rename the existing `mir_supplement_fixture_gate` line in place. Rejected for this slice because downstream consumers may already look for the old line after the call-return docs refresh. Keep the old line until a separate compatibility cleanup retires it.

**Implementation:** Extend `flake.nix` aggregate dependencies/output after adding the focused check.

**Boundary cases:** Aggregate output must keep the non-claim line unchanged and must not imply that MIR extraction is generally supported.

### 3. Keep docs and spec wording selected-fixture-only

**Choice:** Update contributor/current evidence docs only where needed to name MIR inventory coverage and non-claim boundaries.

**Covers:** `Contributor docs name the aggregate command and MIR boundaries`; `MIR fixture inventory lists selected shapes`.

**Rationale:** Operators should discover the rail without reading test code, but docs must avoid overclaiming beyond selected fixtures.

**Alternative considered:** Add broad release/readiness language. Rejected because this is an evidence rail, not a release promotion.

**Implementation:** Patch the aggregate gate paragraph and any evidence index wording that lists aggregate pass lines.

**Boundary cases:** Do not write self-referential final commit hashes into tracked docs.

## Risks / Trade-offs

**Registry drift:** Mitigate with inventory-vs-accepted-selector assertions and inventory-emitted emit/check roundtrip tests.

**Output compatibility:** Mitigate by keeping the old MIR fixture PASS line while adding the inventory PASS line.

**Overclaiming MIR support:** Mitigate with explicit docs/spec non-goals and checks for `unsupported-planned` default behavior.

## Validation Plan

- Focused `cargo test -p cargo-octet mir_fixture -- --nocapture` or a narrower MIR inventory filter.
- Focused Nix `mir-supplement-inventory-gate`.
- Aggregate `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`.
- `openspec validate add-mir-inventory-aggregate-rail --strict --json`.
- `openspec validate --all --strict --json`.
- `git diff --check`.
