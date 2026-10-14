## Why

A function can be pure but still non-deterministic if it builds ordered output from `HashMap` or `HashSet` iteration without sorting. Deterministic cores should produce stable outputs for tests, receipts, evidence, and replay.

## What Changes

- Define a `nondeterministic_output` advisory lint for obvious unordered-collection iteration flowing into ordered outputs.
- Start with local shapes such as `HashMap` / `HashSet` iteration collected into `Vec`, `String`, serialized output, or receipt-like rows without an intervening sort.
- Require guardrails for sorted output, `BTreeMap` / `BTreeSet`, and explicitly unordered return types.

## Impact

- **Files**: `src/purity/nondeterministic_output.rs`, registration, metadata catalog, lint reference, templates, and UI fixtures.
- **Testing**: positive fixtures for unsorted hash iteration and negative fixtures for sorting, ordered maps, and unordered collections.
