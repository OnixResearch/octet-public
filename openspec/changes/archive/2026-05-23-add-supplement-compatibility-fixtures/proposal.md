## Why

Octet now has selected HIR/THIR/MIR supplement replay surfaces. The next drift risk is accepting malformed, future, or boundary-tampered supplement receipts without a deterministic compatibility rail.

## What Changes

- Add a focused supplement compatibility test rail covering a positive v1 supplement receipt and fail-closed negative receipt shapes.
- Add typed invalid JSON reports for supplement checks so malformed receipts return deterministic JSON instead of a raw CLI error when JSON output is requested.
- Compose the focused rail into the aggregate Valence object-evidence gate.

## Scope

In scope:
- Supplement receipt check behavior for HIR/THIR/MIR-style supplement receipts, implemented through the current MIR selected fixture as a stable representative.
- Negative compatibility cases for missing required fields, unsupported schema version, wrong family, malformed JSON, and boundary tamper.
- Aggregate gate wiring and output line.

Out of scope:
- New HIR/THIR/MIR fixture shapes.
- General HIR, THIR, or MIR extraction.
- Correctness, semantic-equivalence, pure-cache, cross-machine, or cross-toolchain claims.

## Impact

- Files: `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, `flake.nix`, OpenSpec artifacts.
- APIs: `cargo octet object supplement check --output-format json` gains deterministic `status = "invalid"` reports for invalid receipts.
- Testing: focused Cargo test, named Nix rail, aggregate Valence gate, OpenSpec validation, and diff hygiene.
