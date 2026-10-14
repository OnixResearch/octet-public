## Why

Octet now has separate verification rails for Function Object v1 corpus replay, compatibility fixtures, caveat taxonomy reporting, and bounded fresh-build receipt repeatability. Operators need one stable Valence-oriented entry point that composes those rails without implying correctness, semantic equivalence, exhaustive Rust coverage, or cross-machine determinism.

## What Changes

- Add a named Nix check for aggregate Valence object evidence.
- Compose the existing object corpus gate, compatibility fixtures, caveat taxonomy report, and fresh-build determinism rail.
- Document the aggregate command as the preferred operator check for object-evidence slices.

## Non-Goals

- No new Rust object identity format.
- No HIR/MIR extraction.
- No Aspen or downstream pilot run.
- No correctness, semantic-equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-machine determinism claim.

## Impact

- Files: `flake.nix`, `docs/contributor-guide.md`, `openspec/specs/standards-codification/spec.md`.
- APIs: new flake check `checks.x86_64-linux.valence-object-evidence`.
- Testing: build the new named flake check and run the normal Octet verification gate.
