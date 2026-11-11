## Why

Octet's object corpus gates now prove fixture replay, compatibility, contract metadata, and fresh-target repeatability for the selected internal corpus. That corpus is still centered on `cargo-octet/src/*.rs`, so it does not yet exercise the broader in-repo standards crate surface that will matter before downstream Valence pilots.

## What Changes

- Add an expanded internal object corpus taxonomy that includes selected `cargo-octet` sources plus stable `crates/slotcar-standards/src/*.rs` modules.
- Add a checked-in expanded corpus receipt fixture separate from the existing core fixture.
- Add replay/freshness tests that prove the expanded fixture remains deterministic and preserves no-correctness/no-semantic-equivalence boundaries.

## Non-Goals

- No Aspen, Crunch, or external downstream pilot.
- No HIR/MIR evidence implementation.
- No claim that hashes prove correctness, semantic equivalence, cross-machine determinism, or exhaustive Rust coverage.

## Impact

- **Files**: object corpus tests and fixtures under `cargo-octet/tests/fixtures/object-corpus/`.
- **APIs**: no public CLI changes.
- **Testing**: focused expanded-corpus tests plus the normal Octet verification rail.
