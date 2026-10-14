## Context

The repository already has `LINT_INVENTORY`, diagnostic metadata snapshots, and fixture coverage checks. Those tests prove registration consistency inside the lint crate, but the README and lint reference still contain hand-maintained totals and domain tables. This change closes that smaller gap without waiting for a full shared metadata architecture.

## Decisions

### 1. Treat Rust registration as the freshness source

**Choice:** Derive expected lint names, default levels, and domain membership from the existing Rust inventory plus diagnostic metadata during tests.

**Rationale:** Runtime registration is the source of truth for shipped lints. This avoids adding another hand-authored catalog while the broader `centralize-lint-metadata` change is still open.

### 2. Check docs deterministically, not visually

**Choice:** Add tests that parse or compare specific README/lint-reference catalog regions for total counts, domain counts, default levels, and lint rows.

**Rationale:** Reviewers need deterministic failure when a new lint is registered but docs are not refreshed. A narrow parser or generated snapshot keeps failures actionable.

### 3. Keep namespace choice out of scope

**Choice:** This change may update namespace prose only to match the live namespace decision, but it does not choose the canonical namespace.

**Rationale:** Namespace policy belongs to `settle-octet-namespace-migration`. Catalog freshness should consume that decision and prevent stale prose after the decision is made.

## Risks / Trade-offs

- Parsing Markdown tables can be brittle. Mitigate by either generating the tables or keeping the parser restricted to stable table shapes.
- This overlaps with `centralize-lint-metadata`. Mitigate by making this the immediate freshness guard and letting the broader change replace the guard's data source later.
- If namespace migration lands first, this change must refresh namespace-dependent docs from the selected canonical behavior.

## Validation

- Positive: current README and lint reference match the inventory-derived total, domain counts, defaults, and summaries.
- Negative: a stale fixture or test input with an incorrect total or missing lint row fails with a diagnostic naming the mismatched surface.
- Regression: targeted docs/catalog tests run with `nix develop -c cargo test --lib` or an equivalent focused docs test rail.
