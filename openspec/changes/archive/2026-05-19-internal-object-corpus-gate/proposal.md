## Why

Octet can emit, replay, and schema-check internal object corpus receipts, but the repository does not yet carry a stable gate fixture that proves the bounded internal corpus remains intentionally reviewed as source changes land.

## What Changes

- Add a checked-in internal object corpus receipt fixture for the bounded `cargo-octet/src/*.rs` corpus.
- Add a CI-friendly test/gate that replays the checked-in fixture with `cargo octet object corpus check --output-format json`.
- Keep the gate scoped to review/provenance evidence identity and explicit no-correctness/no-semantic-equivalence boundaries.

## Out of Scope

- No Aspen/Crunch downstream pilot.
- No HIR/MIR extractor work.
- No claim that corpus receipt matching proves behavioral correctness, semantic equivalence, or cross-machine determinism.

## Verification

Run focused gate tests, save deterministic pass/boundary evidence, then run the maintained Octet verification command through the Nix devshell.
