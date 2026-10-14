## Why

`no_unwrap` design review has repeatedly missed the same-branch guard suppression promised by proposals and specs.

## What Changes

- Require same-branch guard suppression for `.unwrap()` after `.is_some()` / `.is_ok()`.
- Keep unguarded or disjunctive unwraps warning.

## Capabilities

### Modified Capabilities
- `safety`: `no_unwrap` precision for guarded branches.

## Impact

- **Files**: `src/safety/no_unwrap.rs`, `ui_tests/no_unwrap_guarded.rs`.
- **APIs**: no public API changes.
- **Dependencies**: none.
- **Testing**: UI fixture must show same-branch guard suppression and continued warnings for unguarded unwraps.
