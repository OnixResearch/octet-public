## Why

Octet now has a generated current-head Valence evidence page and an aggregate `valence-object-evidence` replay gate, but operators still need a portable release-style bundle/check rail that binds the checked evidence page, the freshness check output, BLAKE3 artifact hashes, and the existing bounded non-claim language. Without that rail, downstream review has to trust scattered repo-local checks instead of a single release evidence package signal.

## What Changes

- Add a named Nix check, `checks.x86_64-linux.valence-evidence-release`, that materializes a bounded Valence evidence bundle with an index receipt, current-head evidence page, freshness-check transcript, and BLAKE3 artifact hashes.
- Wire the new release evidence gate into the existing aggregate `valence-object-evidence` closeout path.
- Keep the scope deliberately narrow: local selected-corpus / selected-fixture evidence replay only, with explicit non-claims for correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, and general HIR/THIR/MIR extraction.

## Impact

- **Files**: `flake.nix`, this Cairn change package, and synced `cairn/specs/valence-evidence/spec.md` after closeout.
- **Testing**: `nix build .#checks.x86_64-linux.valence-evidence-release --no-link -L`, `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`, Cairn validate/gates, and `git diff --check`.
