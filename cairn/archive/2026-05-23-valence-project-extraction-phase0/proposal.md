# Valence Project Extraction Phase 0

## Why

Valence has enough Octet-local evidence rails to justify beginning an extraction path, but a cutover would still be premature because the current guarantees are bounded to selected local corpora and fixtures. Before moving crates or publishing a separate repository, Octet needs a reviewed boundary map that says what Valence owns, what Octet keeps, which Nix/Cairn gates must survive, and which claims remain explicitly out of scope.

## What Changes

- Add a Phase 0 extraction contract for Valence as a future standalone project.
- Record a mechanical inventory of candidate Valence-owned files, Octet-retained seams, and shared/review-only evidence surfaces.
- Define the minimum handoff gates that must exist before any code migration or repository cutover.
- Preserve bounded claim language for all generated evidence and release-style bundles.

## Non-Goals

- No code move, crate rename, package rename, repository creation, tag, or release cut.
- No claim of correctness, semantic equivalence, exhaustive Rust coverage, general HIR/THIR/MIR extraction, cross-machine determinism, or cross-toolchain determinism.
- No downstream pilot migration.
- No credential, signing, remote publication, or CI secret handling.
