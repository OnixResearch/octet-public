# Design: Kani model-check evidence rail

## Approach

The Kani rail is optional external evidence. The CLI shell reads Kani output and command metadata, while the pure receipt core validates a typed in-memory receipt before artifact verification accepts it.

## Receipt shape

A Kani receipt records:

- rail id, schema version, and Octet policy identity;
- package, target, harness path, harness name, and source scope;
- `cargo kani` invocation identity and normalized arguments;
- Kani, rustc, CBMC, solver, and Nix/Onix toolchain identity when available;
- unwind bounds and unwinding assertion status;
- checker verdict, failed property names, and bounded diagnostic summaries;
- function contracts, loop contracts, quantifiers, and contract harnesses used;
- verified-stub and plain-stub inventories;
- BLAKE3 input and receipt hashes;
- explicit non-claims.

## Evidence classification

Passing bounded harnesses are scoped model-check evidence. Passing unwinding assertions or loop/function contracts may upgrade the scoped claim to unbounded for the exact property they discharge. Plain stubs remain boundary assumptions. `stub_verified` is accepted only when the linked contract receipt is present and passing.

## Fixture strategy

Positive fixtures cover a passing bounded harness, a passing contract harness, and a caller using a verified stub. Negative fixtures cover missing tool identity, failed Kani checks, unwinding assertion failure, stale input hash, missing non-claims, and plain-stub promotion.

## What is explicitly not provided

- No starter `cargo octet check` dependency on Kani.
- No whole-program correctness claim.
- No verifier-soundness proof.
- No claim for unmodeled FFI, aliasing models, async, or concurrency.
