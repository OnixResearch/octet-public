## Why

Octet receipts and provenance are already structured around Rust-owned DTOs and Valence exports. Preserves should be available as an optional edge envelope for stack consumers, not as a replacement for Octet's internal evidence model.

## What Changes

- Add an optional Preserves receipt envelope export contract.
- Wrap verified Octet artifact/provenance outputs into canonical Preserves envelopes with schema label, BLAKE3 identity, role, source receipt digest, and non-claims.
- Reject envelopes that bypass artifact verification, promote warning-only evidence to proof evidence, or lose source receipt linkage.

## Impact

- Molten/Mantle/Cairn can consume Octet evidence as `.preserves` carriers when useful.
- Octet keeps local receipt semantics and Valence adapter exports as source-of-truth evidence.
