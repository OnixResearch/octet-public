## Context

Referential transparency requires the same input to yield the same output. Rust hash collections intentionally do not provide stable iteration order, so converting them directly into ordered artifacts weakens replayability.

## Decisions

### 1. Keep the first version advisory

**Choice:** Start `nondeterministic_output` at an advisory default level unless dogfood evidence justifies `warn`.

**Rationale:** Data-flow precision is limited and false positives are possible.

### 2. Match obvious local flow only

**Choice:** Detect direct hash collection iteration followed by `collect::<Vec<_>>()`, string joining, serialization-like calls, or receipt row construction without an intervening sort.

**Rationale:** These are high-signal local patterns and avoid broad whole-function inference.

### 3. Treat sorting and ordered collections as guardrails

**Choice:** Suppress when the path uses `sort`, `sort_by`, `sort_unstable`, `BTreeMap`, `BTreeSet`, or returns an explicitly unordered collection.

**Rationale:** The remediation is stable ordering, not avoiding hash collections entirely.

## Risks / Trade-offs

- Local flow matching may miss multi-step transformations.
- Sorting detection can be too broad if it scans whole blocks; it must stay tied to the output path.

## Validation

Use fixtures that prove both unsorted emitting cases and sorted/ordered non-emitting cases.
