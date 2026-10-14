## Why

external evidence tooling makes its formal claims easy to inspect by presenting named proof cards. Octet has scoped Verus proof rails and receipts, but reviewers currently inspect them through command output, receipts, and prose. A small proof-card surface would make each Verus rail reviewable at a glance while still linking back to the structured receipt and artifact verifier.

Octet should add human-readable proof cards for its Verus rails. Each card should be generated from or checked against the same source-of-truth fields used by proof receipts, so the card is an index into evidence rather than a hand-written overclaim. Cards may also reference external Trellis proof bundles by BLAKE3 identity, but those imported references remain external/manual-review evidence unless Octet reruns the proof with a pinned verifier and validates an Octet-owned receipt.

## What Changes

- Add a proof-card documentation or artifact surface for Verus rails.
- Each card names the proof/property ID, source scope, proof target, verifier mode, verifier/solver/toolchain identity, receipt path, artifact BLAKE3 identities, evidence tier, status, assumptions/trust boundaries, and non-claims.
- Cards link to or are validated against the structured proof receipts and `cargo octet artifact verify` behavior.
- Imported Trellis cards name the Trellis repository revision, source/proof/log/receipt BLAKE3 identities, and the external-review tier unless Octet re-executes the proof rail.
- Add positive and negative checks so cards cannot omit required fields or claim more than the receipt supports.

## Non-Goals

- Do not create a new proof system or replace structured proof receipts.
- Do not add new Verus proof targets as part of this presentation change.
- Do not treat proof cards as standalone evidence without the matching receipt/provenance bundle.
- Do not treat a BLAKE3 hash as proof correctness; it only identifies exact artifact bytes.
- Do not require proof cards for ordinary starter `cargo octet check` runs with no selected proof rails.

## Impact

- **Files**: `verus/README.md`, proof docs, a proof-card fixture or generated artifact path, proof-card validation tests/helpers, README references, and possibly imported Trellis bundle fixtures.
- **Behavior**: review presentation only unless a selected command explicitly emits proof cards; no change to proof execution semantics.
- **Testing**: positive card inventory for current fixture and production pilots; negative fixtures for missing required fields, stale receipt links, and unsupported broad proof claims.
