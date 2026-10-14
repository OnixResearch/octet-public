# Design: Valence in-repo core crate

## Boundary

`valence-core` is the first in-repository Valence-owned crate. It is a pure deterministic core crate: it accepts strings/DTOs and returns strings/reports. It does not read files, inspect git state, invoke Nix, print to stdout/stderr, or own CLI parsing.

`cargo-octet` remains the imperative shell. Its `evidence-index` command parses arguments, reads the selected Markdown page, prints pass/fail lines, and delegates deterministic render/check behavior to `valence-core`.

## First extracted seam

The evidence-index renderer/checker is selected because it is already deterministic, directly tied to the Valence release/evidence gates, and small enough to move without broad object-model churn. The generated page text and fail-closed issue codes remain compatibility surfaces.

## Verification

Verification must demonstrate:

- `valence-core` tests cover positive render/check roundtrip and negative fail-closed cases.
- `cargo-octet` still has an evidence-index CLI shim test proving it delegates to the core renderer/checker.
- Existing command behavior still validates `docs/current-head-valence-evidence.md`.
- Named Nix gates `valence-evidence-index`, `valence-evidence-release`, and `valence-object-evidence` continue to pass.
- Cairn validation and gates pass.

## Non-claims

This slice proves an in-repo crate seam for selected deterministic evidence-index logic only. It does not prove standalone repository readiness, published package consumability, correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.
