# Valence core package readiness rail

## Why

Phase 1 and Phase 2 moved selected pure Valence evidence logic into the in-repository `valence-core` crate. Before any standalone repository or package cutover, the next extraction step is to add a first-class readiness rail that proves the crate has package metadata, public API boundary documentation, and a directly buildable Nix check while Octet remains the repository, CLI, filesystem, and release shell.

## What changes

- Add package-consumability metadata and a crate README for `valence-core`.
- Expose a buildable `packages.${system}.valence-core` output.
- Add a named `checks.${system}.valence-core-package-readiness` Nix rail that asserts the package builds, package metadata is present, public modules remain exposed, and README boundary/non-claim language is present.
- Wire the readiness rail into the aggregate `valence-object-evidence` check.
- Preserve Cairn validation and the existing evidence-index/release/object evidence gates.

## Out of scope

- No standalone repository creation.
- No crates.io publication, tag movement, release ownership transfer, or public release cutover.
- No migration of Octet CLI/filesystem/scanner/supplement/corpus shell behavior into `valence-core`.
- No correctness, semantic-equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction claims.
