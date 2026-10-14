## Why

Randomness is hidden input. Pure functions should receive seeds, RNG traits, generated IDs, or nonce bytes explicitly so tests and proofs can replay behavior deterministically.

## What Changes

- Define an `ambient_random` lint for direct randomness and non-deterministic ID generation in production/core code.
- Start with known direct calls such as `rand::thread_rng`, `rand::rng`, `getrandom`, `OsRng`, and UUID v4 constructors.
- Require remediation guidance toward injected RNGs, explicit seeds, or shell-generated values.

## Impact

- **Files**: `src/purity/ambient_random.rs`, registration, metadata catalog, lint reference, templates, and fixtures.
- **Testing**: positive fixtures for direct RNG/UUID generation and negative fixtures for injected RNG traits or deterministic seeds.
