## Context

Allow-by-default lints are useful for inventory and targeted rollout, but promotion changes consumer experience. The promotion path needs objective gates so maintainers can see why a lint is ready for `warn`, starter profiles, or `deny` templates.

## Decisions

### 1. Promotion requires evidence, not preference

**Choice:** A promotion package must include dogfood counts, downstream sample counts when available, false-positive triage, remediation examples, and suppression compatibility proof.

**Rationale:** Default levels are product contracts.

### 2. Separate default-level promotion from profile promotion

**Choice:** A lint can enter a starter/boundary profile before its global default changes, and the evidence must state which surface changes.

**Rationale:** Profiles let consumers adopt high-signal checks without forcing all users into a new default.

### 3. Keep migrations explicit

**Choice:** Any promoted lint must update README/lint-reference/template docs, default-level snapshots, and downstream rollout notes with expected migration behavior.

**Rationale:** Consumers need a clear diff and rollback path.

## Candidate first wave

- `raw_arithmetic_overflow`
- `ambiguous_params`
- `multi_lock_ordering`
- `defaulting_trait_method`
- `numeric_units`

## Validation

Run default-level snapshot tests, UI fixtures under old and new levels, config override tests, dogfood profile reruns, and Cairn validation/gates.
