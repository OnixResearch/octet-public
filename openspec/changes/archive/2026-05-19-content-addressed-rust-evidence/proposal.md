## Why

The content-addressed Rust evidence work belongs primarily in Octet because Octet owns Rust source analysis, function fingerprints, review markers, diffs, and exportable evidence surfaces. Cairn can later consume Octet receipts, but Octet must define and implement the product-facing object model first.

Current Octet function fingerprints are useful review gates, but downstream Valence, Trellis, Crunch/Mantle, and Aspen integrations need a stronger Octet-native contract for deterministic function objects, name pointers, caveats, bundle export, and future compiler-IR evidence.

## What Changes

- Add an Octet-owned OpenSpec capability for content-addressed Rust evidence objects.
- Define function-object v1 as a source-token, review/provenance identity object with canonical BLAKE3 hashing and explicit caveats.
- Define mutable name pointers to immutable object hashes, object diffs, marker compatibility, and export bundles.
- Define HIR/MIR as future supplemental compiler-grounded evidence, not the first milestone.
- Define downstream interop boundaries for Cairn receipts, Trellis admission, Crunch/Mantle evidence rows, and Aspen CI/job evidence.

## Non-Goals

- Build a global Unison-for-Rust runtime.
- Claim behavioral correctness or semantic equivalence from matching hashes.
- Require HIR/MIR extraction for v1.
- Replace Cargo, Git, Cairn lifecycle policy, Crunch/Mantle builds, Aspen CI, or Trellis proofs.

## Impact

- **Files**: OpenSpec change under `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/`; later implementation will primarily touch `cargo-octet/src/fingerprint.rs` or adjacent object modules, docs, templates, and fixtures.
- **APIs**: Future `cargo octet object ...` commands and JSON bundle schemas.
- **Testing**: OpenSpec validation/gates now; later docs/examples checks plus focused object canonicalization, marker, pointer, diff, and bundle tests.
