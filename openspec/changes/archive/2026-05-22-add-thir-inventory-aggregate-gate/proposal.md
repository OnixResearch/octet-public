## Why

The aggregate Valence object-evidence gate is the operator-facing check for selected Octet evidence rails. THIR now has fail-closed supplement support, two selected fixture shapes, and a read-only fixture inventory with replay commands. That inventory should be covered by the aggregate gate so CI/operator runs catch selector drift and boundary regressions automatically.

## What Changes

- Add a THIR fixture inventory rail to the aggregate Valence object-evidence gate.
- Keep the THIR inventory scope closed to `checked-arithmetic` and `call-return`, with `branch-control-flow` HIR-only.
- Preserve default/non-fixture THIR `unsupported-planned` metadata and non-proof boundaries.

## Out of Scope

- No third THIR fixture shape.
- No general THIR extraction or semantic correctness claims.
- No broad aggregate gate redesign beyond composing the new THIR inventory rail.

## Impact

- Files: `flake.nix`, `docs/contributor-guide.md`, `cargo-octet/tests/fingerprint_consumer.rs`, and `openspec/specs/standards-codification/spec.md`.
- APIs: one new named Nix check may be exposed for focused THIR inventory rail verification; the existing aggregate check composes it.
- Verification: focused Cargo tests, focused Nix check, aggregate Nix check, OpenSpec gates/validation, and diff hygiene.
