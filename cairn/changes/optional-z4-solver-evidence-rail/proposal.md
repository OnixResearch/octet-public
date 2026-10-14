## Why

Octet already owns opt-in formal proof rails, external evidence receipts, proof cards, Valence exports, and non-claim boundaries. The z4 repository is a useful reference for a Rust-native SMT/CHC solver API, conservative `unknown` behavior, proof artifacts, progress JSON, and solver diagnostics. Octet should adapt those ideas as an optional experimental evidence rail without replacing Verus, Kani, or existing solver trust boundaries.

Reference source: `https://github.com/joweeba/z4`.

## What Changes

- Define an optional `z4-solver` evidence rail profile for already-produced SMT/CHC/SAT proof or diagnostic artifacts.
- Record solver identity, input query identity, expected logic, result status, proof/model/core artifact refs, timeout/unknown reason, validation policy, and non-claims.
- Add positive and negative fixtures for `sat`, `unsat-with-proof`, `unknown`, stale proof, unsupported logic, wrong solver identity, and overclaiming replacement claims.
- Keep Z4 execution outside starter `cargo octet check`; rail collection remains explicit and recorded-only unless later promoted.

## Impact

- **Surfaces**: optional evidence collect rail, proof artifact library, proof-card review, Valence export boundaries.
- **Non-claims**: No Z3 replacement, no solver soundness proof, no proof discharge unless independently checked by a scoped rail.
- **Validation**: rail fixture tests, artifact verification, proof-card validation, Cairn gates/validate.