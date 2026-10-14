## Why

Octet already defines Function Object v1 as the canonical source-token review/provenance identity and exposes HIR/MIR supplement families as `unsupported-planned` in contract metadata. The aggregate Valence evidence gate is green, so the next product step is a bounded pilot that proves how toolchain-scoped compiler-IR supplements attach to existing object evidence without overclaiming correctness, semantic equivalence, or cross-toolchain stability.

Without a pilot contract, later HIR/MIR work risks jumping from roadmap metadata directly to ad hoc extractor output. This change keeps the first implementation slice narrow, fixture-backed, and fail-closed.

## What Changes

- Define a bounded HIR/MIR supplement pilot for selected Octet fixture functions.
- Require supplement records to reference an existing `octet.function-object.v1` object hash and exact rustc/toolchain identity.
- Add typed pilot receipt/check-report expectations for generated supplement evidence.
- Promote contract metadata from `unsupported-planned` to `pilot-supported` only for families with deterministic fixture evidence; unimplemented families remain fail-closed.
- Preserve existing non-proof language: supplements improve review evidence, not correctness or semantic-equivalence claims.

## Non-Goals

- Full workspace-wide HIR/MIR extraction.
- Cross-toolchain or cross-optimization equivalence claims.
- Replacing Function Object v1 source-token identity.
- Proving behavioral correctness, safety, semantic equivalence, or pure-cache eligibility.
- Downstream Cairn/Trellis/Crunch/Aspen policy integration beyond keeping Octet receipts consumable.

## Impact

- **Files**: Future implementation likely touches `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, object-corpus fixtures, `docs/content-addressed-rust-evidence-boundaries.md`, and `docs/contributor-guide.md`.
- **APIs**: Future `cargo octet object ...` or `cargo octet object corpus ...` subcommands may gain bounded supplement emit/check surfaces and contract metadata statuses.
- **Testing**: Focused cargo-octet tests, checked-in positive/negative supplement fixtures, contract metadata checks, `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`, strict OpenSpec validation.
