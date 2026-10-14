## Why

`docs/current-head-valence-evidence.md` is currently a hand-edited index over Octet/Valence evidence rails. The page records a verified payload commit, a command sweep, expected aggregate pass lines, and explicit non-claims, but those fields can drift after future Valence fixture or receipt slices. A first-class generator/staleness check keeps the evidence index reproducible and prevents stale manual refreshes.

## What Changes

- Add an Octet-owned Valence current-head evidence index generator that renders the Markdown index from deterministic repository state, configured rail inventory, and captured verification results.
- Add a fail-closed staleness check that compares the generated index with `docs/current-head-valence-evidence.md` and reports mismatched payload commit, rail list, pass signal, or non-claim boundaries.
- Wire the check into local verification so future evidence slices can prove the index is fresh without hand-auditing every field.
- Preserve current boundaries: the generator records bounded local selected-corpus / selected-fixture evidence only; it does not claim correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, or cross-toolchain determinism.

## Impact

- **Files**: expected implementation areas are `cargo-octet` tests or a small Rust helper, `flake.nix` check wiring, and `docs/current-head-valence-evidence.md` output contract.
- **Testing**: generated-doc equality check, stale/mismatched negative fixtures, `valence-object-evidence`, Cairn validation/gates, and `git diff --check`.
