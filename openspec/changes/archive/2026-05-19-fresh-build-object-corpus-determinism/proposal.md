## Why

Octet now has object corpus receipts, schema compatibility fixtures, an expanded internal corpus taxonomy, and a named object corpus gate. The remaining internal evidence gap is showing that a freshly built Octet binary can regenerate the selected object corpus evidence deterministically, rather than relying only on the binary already under `cargo test`.

## What Changes

- Add a Nix-backed fresh-build object corpus determinism rail that builds separate `cargo-octet` packages and compares their generated corpus receipts.
- Compare deterministic receipt bytes and stable contract fields against the expanded internal corpus fixture.
- Preserve explicit evidence boundaries: this is bounded local fresh-build receipt repeatability, not correctness, semantic equivalence, or cross-machine proof.

## Impact

- Files: `flake.nix`, object corpus fixtures/tests if needed, OpenSpec standards-codification spec.
- APIs: no CLI API changes.
- Testing: run the new flake check plus the standard Octet verification rail.
