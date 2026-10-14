## Why

Rust's `Deref` pattern is valuable for smart pointers, string/slice wrappers, and narrow transparent accessors, but the Rust design-pattern guidance warns against using `Deref` as an inheritance or polymorphism mechanism. Public `Deref<Target = SomeDomainType>` implementations silently import another type's method surface and can blur Tiger Style API boundaries.

Tigerstyle already documents this guidance. A narrow advisory lint can surface the highest-signal cases: public local types implementing `Deref` to another nominal local/domain type, while accepting private wrappers and primitive borrowed targets like `str` and `[T]`.

## What Changes

- Add advisory lint `tigerstyle::deref_polymorphism` in the structure domain.
- Warn on public local `impl Deref` surfaces whose `Target` is a nominal type rather than `str` or a slice.
- Keep the lint diagnostic-only and default-warn for staged rollout.
- Update docs, templates, fixture inventories, and UI evidence.

## Out of Scope

- No blanket ban on `Deref`.
- No warning for private implementation details.
- No warning for accepted borrowed-view targets such as `str` and `[T]`.
- No hard-gate promotion without future downstream evidence.

## Verification

Verify with focused UI fixtures, inventory tests, canonical OpenSpec validation, policy validation, and diff whitespace checks.
