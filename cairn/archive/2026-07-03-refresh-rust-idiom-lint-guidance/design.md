## Context

Catalog freshness already protects the main README and lint reference tables. Guidance pages are more narrative and can still drift when they describe whether a pattern is advisory, future work, or an implemented lint.

## Decisions

### 1. Fix the known stale guidance immediately

**Choice:** Update `Borrowed argument type` in `docs/rust-idiom-guidance.md` to state that `borrowed_argument_types` is shipped as a warning lint with positive/negative fixtures.

**Rationale:** The current wording says it is a future candidate, which is false.

### 2. Add a small guidance-freshness check

**Choice:** Add a deterministic test or fixture that verifies named shipped lints in guidance pages are not described as future-only candidates.

**Rationale:** The check should prevent this exact drift class without turning narrative docs into generated tables.

### 3. Keep broader prose human-authored

**Choice:** Do not generate the whole guidance page from metadata. Only validate rows that explicitly reference shipped lint identities or enforcement tiers.

**Rationale:** Guidance docs carry review context that does not belong in the lint catalog.

## Risks / Trade-offs

- A too-broad prose parser could be brittle. Keep the first check exact and fixture-backed.
- Some future-candidate rows are legitimate. The check only applies to lint identities present in the live inventory.

## Validation

Run the docs freshness test, `cargo test --lib`, and Cairn validation/gates.
