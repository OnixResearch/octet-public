## Why

`evidence_codebase_index` is now implemented inside `valence-core`, but its model, CLI help, HTML title, and spec describe an Octet evidence codebase index. That may be a useful shared pure core, yet the current seam does not say whether the capability is Valence-owned, Octet-owned-but-hosted, or ready for a separate shared crate. The module is also large enough that model, ingestion, query, HTML, and boundary logic are harder to review together than they need to be.

## What Changes

- Clarify the ownership boundary for evidence-codebase indexing while it is hosted in `valence-core`.
- Keep generated output explicit that the index is an Octet evidence index unless a future reviewed change parameterizes or moves product identity.
- Decompose the core implementation into reviewable pure submodules without changing CLI behavior.
- Add readiness/compatibility checks that prove generated JSON/HTML boundaries, public API compatibility, and query behavior stay deterministic.

## Impact

- **Files**: `crates/valence-core/src/evidence_codebase_index.rs` and submodules, `crates/valence-core/README.md`, `docs/evidence-codebase-index.md`, `cargo-octet/src/evidence_index.rs`, `cairn/specs/evidence-codebase-index/spec.md`, and possibly `cairn/specs/valence-project-extraction/spec.md` for seam notes.
- **Testing**: focused `valence-core` evidence-codebase index tests, `cargo-octet` evidence-index codebase command tests, the named `evidence-codebase-index` Nix rail, `valence-core-package-readiness`, and Cairn validation/gates.
