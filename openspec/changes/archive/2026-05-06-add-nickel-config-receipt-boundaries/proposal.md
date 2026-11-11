## Why

Tiger Style already uses typed Nickel contracts for policy configuration and generated proof artifacts, but the boundary for future config and receipt surfaces is implicit. Without an explicit source-of-truth rule, human-authored run knobs, generated machine inputs, Rust-emitted receipts, and raw logs can drift into duplicate schemas or unreviewable acceptance records.

## What Changes

- Define a registry-backed decision model for config and receipt families: `nickel-authored`, `rust-derived`, or `non-candidate`.
- Require human-authored configuration to live in Nickel contracts and export deterministic machine formats for tools.
- Require Rust-owned receipt DTOs to generate Nickel validation contracts instead of hand-maintained duplicate schemas.
- Exclude secrets, cryptographic internals, runtime behavior, wire discriminants, hot-path constants, and raw logs from Nickel ownership.
- Add verification obligations for registry freshness, generated-contract drift, positive/negative fixtures, and Rust serialization round trips.

## Capabilities

### Modified Capabilities
- `standards-codification`: standards policy artifacts gain explicit Nickel-vs-Rust source-of-truth rules for configuration and proof/evidence receipts.

## Impact

- **Files**: `standards/policies/*.ncl`, future `standards/schemas/*.ncl`, receipt/config generators, standards docs, fixture tests, and OpenSpec evidence.
- **APIs**: No immediate runtime API change; future receipt/config surfaces must declare ownership before implementation.
- **Testing**: Nickel typecheck/export, generated-contract freshness checks, positive/negative Nickel fixtures, Rust serde/schema round trips, registry invariant checks, OpenSpec validation, and whitespace checks.
