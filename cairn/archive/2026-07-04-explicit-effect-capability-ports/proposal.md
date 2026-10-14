## Why

Haskell makes effects visible in types. Octet should adapt that lesson by keeping platform effects out of pure cores and passing explicit inputs, capabilities, or shell-owned results instead. This lines up with existing functional-core rules and purity-effect lints, but the runner/core architecture needs a clearer implementation target for effect seams.

The goal is not a Rust effect system. It is a practical contract: filesystem, environment, process, time, randomness, logging, metrics, network, and async-runtime access belong in thin shells. Pure cores receive typed data and return decisions, diagnostics, events, or plans.

## What Changes

- Add explicit runner/core architecture requirements for effect capability seams.
- Define small, law-like Rust traits or data tokens for shell-injected capabilities where direct values are not enough.
- Keep broad service traits, hidden globals, ambient reads, and mock-heavy pure-core tests out of the design.
- Add positive and negative tests proving pure cores can be exercised without filesystem, process, environment, clocks, randomness, or observability side effects.

## Impact

- **Files**: `cargo-octet/src/` runner cores and shells, standards checker seams, selected `crates/valence-core` hosted cores, docs that describe core/shell boundaries, and tests around capability injection.
- **Testing**: pure unit tests without side effects, shell tests with explicit fake capabilities or captured command results, negative scans for ambient effects in core scopes, existing runner integration tests, and Cairn validate/gates.
