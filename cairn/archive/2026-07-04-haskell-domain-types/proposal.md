## Why

Octet already values functional-core boundaries, but many review surfaces still pass important concepts as raw strings, path buffers, numeric counters, or boolean combinations. The Haskell lesson worth adopting is not clever abstraction; it is algebraic domain modeling that makes invalid states hard to express and cheap to test.

Typed domain identities, sum-type state machines, and refined constructors would reduce stringly typed plumbing around lint names, diagnostic identities, evidence families, artifact paths, thresholds, and config state. This should be done incrementally so existing wire formats, CLI output, and Dylint compatibility stay stable.

## What Changes

- Inventory raw domain identities that cross non-trivial Octet core/module boundaries.
- Introduce small Rust newtypes, enums, and refined constructors for high-risk identities and state combinations.
- Preserve existing serialized DTOs, CLI output, source attributes, and config keys through boundary adapters.
- Add positive and negative tests that prove invalid states, empty refined collections, malformed identities, and invalid thresholds fail before reaching core logic.
- Document an abstraction boundary that rejects Haskell-style cleverness: no lens-heavy APIs, no monad terminology, no laziness assumptions, and no type-level machinery without a direct bug-prevention payoff.

## Impact

- **Files**: likely `src/config.rs`, `src/diagnostics.rs`, `src/lib.rs`, `crates/octet-lint-metadata`, selected `cargo-octet/src/` cores, standards registry/checker DTOs, and tests for the migrated seams.
- **Testing**: focused unit tests for constructors and adapters, negative malformed-input tests, config/diagnostic tests, selected runner tests, and Cairn validate/gates.
