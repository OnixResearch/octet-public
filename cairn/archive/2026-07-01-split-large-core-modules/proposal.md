## Why

Several Octet runner and standards files have grown into broad orchestration modules. Large files such as `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/src/prove.rs`, `tests/consumer_integration.rs`, and large fixture tests make review expensive and obscure the functional-core / imperative-shell boundary that Octet promotes for consumers.

Splitting these hotspots into pure cores and thin shells will make behavior easier to test, reduce regression risk, and make future evidence and proof-path work less coupled to CLI plumbing.

## What Changes

- Define module-boundary rules for large runner, proof, object, and integration-test files.
- Extract deterministic parsing, normalization, validation, and rendering logic into pure modules with direct positive and negative tests.
- Keep filesystem, process execution, environment reads, and stdout/stderr in small shell modules.
- Move bulky fixture builders and replay helpers out of monolithic test files into reusable test-support modules.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/src/prove.rs`, `tests/consumer_integration.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, and new focused modules under `cargo-octet/src/` or test support.
- **Testing**: focused unit tests for extracted cores, existing integration tests, UI rails where affected, and a smoke check proving CLI outputs stay stable.
