# Change: Kani model-check evidence rail

## Why

Kani provides a cargo-native model-checking path for Rust panic-freedom, unsafe-boundary safety checks, and contract-backed functional properties. Octet should be able to collect that evidence without making starter `cargo octet check` heavier or letting bounded checks overclaim.

## What Changes

- Add an opt-in Kani model-check evidence rail to `cargo octet evidence collect`.
- Record harness, package, solver, Rust/Kani/CBMC identity, unwind policy, contract use, stub use, status, diagnostics, and explicit non-claims in a deterministic receipt.
- Distinguish bounded proof harnesses, passing unwinding assertions, contract verification, verified stubs, and plain trusted/environment stubs.
- Add positive and negative fixtures for passing harnesses, failed checks, missing identity fields, unwinding assertion failures, stale inputs, and unverified-stub promotion attempts.

## Impact

- **Files**: external evidence rail registry, receipt DTOs/contracts, collector, artifact verifier, fixtures, and operator docs.
- **Consumers**: Valence and Cairn can consume scoped Kani facts without treating them as whole-program proofs.
- **Non-claims**: The rail records Kani results for declared scopes only; it does not prove verifier soundness, whole-program correctness, concurrency correctness, or unmodeled FFI behavior.
