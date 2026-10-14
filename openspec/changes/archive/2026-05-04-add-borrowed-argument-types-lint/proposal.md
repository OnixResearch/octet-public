## Why

The archived Rust-patterns/dynamic-evidence change classified borrowed argument types as a high-ROI Tigerstyle idiom, but it intentionally stopped at docs and policy metadata. The next smallest lint candidate is borrowed argument types because it has a narrow static signal, maps directly to API explicitness and avoidable allocation, and can be proven with compact UI fixtures before any hard-gate promotion.

## What Changes

- Add an OpenSpec for a future `borrowed_argument_types` structure lint.
- Scope the first implementation to owned function/method signatures where ownership is not requested but the argument type is an owned standard container reference such as `&String`, `&Vec<T>`, or `&PathBuf`.
- Require positive, negative, and false-positive-boundary fixtures before the lint can become a default hard gate.
- Keep the lint advisory/allow-by-default until fixture evidence and downstream inventory prove low false-positive behavior.

## Capabilities

### Modified Capabilities
- `structure`: Adds a focused structural API lint candidate for borrowed argument types.
- `standards-codification`: Consumes the previously classified Rust idiom through a child implementation OpenSpec.

## Impact

- **Files**: Future implementation is expected under `src/structure/`, `src/lib.rs`, diagnostics/inventory metadata, UI fixtures, snapshots, and docs/catalog entries.
- **APIs**: No public runtime API change is intended; the lint adds advisory diagnostics to `cargo-tigerstyle`.
- **Dependencies**: No new dependency is expected.
- **Testing**: Validate this OpenSpec now; later implementation must run focused UI tests plus package checks and save evidence under this change.
