## Why

Tigerstyle already has artifact-backed structural gates for bounded queues and effect capabilities. The next weak seam is protocol/state APIs: the policy registry names type-state as a structural target, but the current checker still relies on a small hard-coded raw-pattern list and does not export deterministic type-state obligations through generated policy artifacts.

## What Changes

- Promote `tigerstyle.idioms.nominal-type-boundaries` from advisory/lint metadata to a hard structural gate for declared type-state protocol surfaces.
- Extend Nickel mechanism contracts and generated artifacts with type-state obligations: raw bypass patterns, required state marker names, transition method names, and source scopes.
- Teach `tigerstyle-standards` to consume those generated obligations and fail declared source roots that expose configured raw protocol bypasses while accepting nominal marker-state transition APIs.
- Add Tigerstyle-local compliant and noncompliant fixtures plus deterministic evidence transcripts.

## Out of Scope

- Full Rust semantic/type analysis for arbitrary protocol APIs.
- Downstream Aspen or other consumer lock updates.
- Verified proof rails for protocol transitions; this change is structural, not verified.

## Impact

- **Files**: `standards/policies/`, `crates/tigerstyle-standards/src/`, standards fixtures, docs, and OpenSpec evidence.
- **APIs**: Generated policy artifacts add type-state checker fields consumed by `tigerstyle-standards check --policy-artifacts-dir`.
- **Testing**: Nickel validation, Rust unit tests, standards fixtures, generated artifact export assertions, docs examples, OpenSpec validation, and whitespace checks.
