# Valence core cutover

<!-- r[impl octet.valence_project_extraction.phase5.consumer_migration_docs] -->
<!-- r[verify octet.valence_project_extraction.phase5.hardened_upstream_dependency] -->
<!-- r[verify octet.valence_project_extraction.phase5.exact_source_pin] -->

Octet consumes standalone Valence as the canonical owner of Valence pure evidence semantics. Production calls route through `cargo-octet/src/valence_core_shim.rs` to this exact dependency:

```toml
valence-core = { git = "https://github.com/OnixResearch/valence-public.git", rev = "99cbe41f742035c37b6a7b503e4748e2e5ce2bea", package = "valence-core" }
```

The selected revision contains `cairn/archive/2026-07-12-harden-preserves-integrity-boundaries`, including the corrected Preserves integrity and typed claim-boundary profile. The Nickel-authored cutover manifest at `standards/valence-core-cutover/manifest.ncl` binds that revision, Cargo source identity, archive path, module source hashes, fixture commands, and non-claims. Its generated runtime projection is `standards/valence-core-cutover/generated-manifest.json`.

This independently rooted public export retains the historical archive labels.
Its sanitized evidence-index module has a new measured source hash; the current
manifest binds those public bytes, while frozen legacy compatibility fixtures
remain unchanged. Source availability and matching metadata do not renew the
archived check results or establish release admission.

The old Octet-hosted source remains only at `crates/valence-core/` as the non-publishable, non-workspace package `octet-valence-legacy-fixture`. It is a bounded positive/negative compatibility corpus and cannot satisfy a canonical `valence-core` dependency.

## Consumer migration

Downstream consumers such as Aspen must replace the Octet-hosted package with the exact standalone dependency above. A local `../valence/crates/valence-core` path may be used only as an explicit development diagnostic; it is not release evidence and must not change the manifest or lock identity.

After changing the dependency, run:

```bash
nix develop -c cargo test -p cargo-octet valence_cutover -- --nocapture
nix develop -c cargo test -p cargo-octet evidence_index -- --nocapture
nix develop -c cargo test -p cargo-octet object -- --nocapture
```

Passing these checks proves selected dependency/source ownership, unique canonical package identity, and compatibility for the declared Function Object, evidence-index, and evidence-codebase-index fixtures only. It does not prove behavioral correctness, arbitrary semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, release eligibility, or whole-program safety.
