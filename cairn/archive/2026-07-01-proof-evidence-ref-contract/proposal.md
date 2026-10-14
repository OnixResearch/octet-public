## Why

Octet's formal-proof lane already emits proof cards and validated `formal_proof` provenance, but ordinary source comments, suppressions, and docs do not yet have a small Rust-owned reference shape for saying "this local claim is backed by that scoped proof card." A Rust trait would be the wrong abstraction because consumers could implement it manually and accidentally imply stronger guarantees than the receipt chain supports.

Octet should add a data/evidence contract instead: a `ProofEvidenceRef` record that links a local claim or suppression to a proof card by property ID, checked scope, expected evidence tier, required artifacts, and explicit non-claims. The CLI should validate that reference against `proof-cards.json` without rerunning Verus.

## What Changes

- Add a Rust-owned `ProofEvidenceRef` DTO with a deterministic JSON schema version.
- Add a `cargo octet proof-ref validate --proof-cards PATH --ref PATH` command that validates a reference against a proof-card deck.
- Require exact linkage for proof/property ID, source requirement or policy ID, checked scope, expected evidence tier, status, required artifact identities, and non-claims.
- Keep the reference as evidence linkage only; it does not introduce a Rust trait, source attribute, or standalone proof claim.

## Non-Goals

- Do not add a `Verified` trait or any consumer-implementable proof marker.
- Do not rerun Verus from the proof-ref command.
- Do not change proof receipt generation or proof-card generation in this slice.
- Do not automatically waive lints yet; suppression-to-proof linkage can consume this DTO in a later change.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/lib.rs`, new proof-ref module/tests, docs/help text, and Cairn spec delta.
- **Behavior**: new read-only CLI validation surface for proof evidence references.
- **Testing**: positive ref/card match, negative missing card, negative mismatched scope/tier/status/artifact/non-claim, CLI parse/help coverage, `cargo test -p cargo-octet proof_ref`, and Cairn validation/gates.
