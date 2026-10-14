## Why

Recent Aspen-side consumer review turned up code shaped like
`format!("recent_{}", u64::try_from(command_index).unwrap_or(u64::MAX))`.
That pattern converts missing or invalid state into a fake-but-valid-looking
identifier component. Tiger Style wants invalid state modeled explicitly, not
smuggled through an extreme sentinel that downstream code may accidentally
accept as real.

## What Changes

- Add a new safety lint, `sentinel_fallback`, for `Option` / `Result`
  `unwrap_or` and `unwrap_or_else` calls that replace absence or failure with an
  integer extreme sentinel such as `u64::MAX`, `usize::MAX`, or `i64::MIN`.
- Keep first-pass scope narrow: only same-type integer `MIN` / `MAX` fallback
  constants, production code only, no broad magic-number detection.
- Emit guidance to branch explicitly, propagate the failure, or use a
  domain-tagged non-numeric representation when the code is building names or
  identifiers.

## Non-Goals

- General magic-number linting.
- Automatic fixes; the correct rewrite depends on domain intent.
- Proving protocol-specific reserved sentinels safe in this first pass.

## Capabilities

### New Capabilities
- `sentinel_fallback`: detects `unwrap_or(<int>::MAX|MIN)` and
  `unwrap_or_else(|_| <int>::MAX|MIN)` on `Option` / `Result` values.

## Impact

- **Files**: `src/safety/sentinel_fallback.rs`, `src/lib.rs`,
  `src/diagnostics.rs`, UI fixtures and snapshots, and rollout docs if the new
  lint is exposed in inventory listings.
- **APIs**: one new warn-by-default lint ID, `sentinel_fallback`, with normal rustc allow / deny handling.
- **Dependencies**: none.
- **Testing**: ignored UI coverage for positive and negative cases, including
  explicit branching and non-extreme defaults.
