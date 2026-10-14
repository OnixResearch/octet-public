# Valence core cutover

Octet treats standalone Valence as the canonical owner for Valence-owned pure evidence semantics. The current Octet implementation routes Function Object v1, evidence-index, and evidence-codebase-index calls through an explicit `valence_core_shim` while the dependency manifest records the standalone target and compatibility fixtures.

The manifest is Nickel-authored at `standards/valence-core-cutover/manifest.ncl`. It names:

- standalone target: `github:OnixResearch/valence?dir=crates/valence-core`;
- development override: `../valence/crates/valence-core` only for local review, not release evidence by itself;
- shimmed Octet modules: `function_object`, `evidence_index`, and `evidence_codebase_index`;
- fixture rows for Function Object v1, evidence-index, and evidence-codebase-index compatibility;
- explicit non-claims for correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, and release eligibility.

## Consumer migration

Downstream users that currently depend on Octet's git package `valence-core` should migrate to standalone Valence:

```toml
valence-core = { git = "https://github.com/OnixResearch/valence", package = "valence-core" }
```

Local development may temporarily use a path override to `../valence/crates/valence-core`, but release evidence must pin a reviewed standalone Valence revision or record a reviewed override rationale.

Aspen migration target: replace the Octet-hosted `valence-core` package source with standalone Valence, then run the compatibility checks below in the consumer branch before promotion.

## Validation

Focused Octet-side compatibility:

```bash
nix develop -c cargo test -p cargo-octet valence_cutover -- --nocapture
nix develop -c cargo test -p cargo-octet evidence_index -- --nocapture
nix develop -c cargo test -p cargo-octet object -- --nocapture
nix develop -c cargo test -p valence-core function_object -- --nocapture
nix develop -c cargo test -p valence-core evidence_codebase_index -- --nocapture
nix develop -c cargo test -p valence-core evidence_index -- --nocapture
```

Passing these checks proves dependency/source ownership and DTO compatibility only. It does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, release eligibility, or whole-program safety.
