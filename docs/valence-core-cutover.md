# Valence core cutover

<!-- r[impl octet.valence_project_extraction.phase5.consumer_migration_docs] -->
<!-- r[verify octet.valence_project_extraction.phase5.hardened_upstream_dependency] -->
<!-- r[verify octet.valence_project_extraction.phase5.exact_source_pin] -->

Octet consumes standalone Valence as the canonical owner of Valence pure evidence semantics. Production calls route through `cargo-octet/src/valence_core_shim.rs` to this exact dependency:

```toml
valence-core = { git = "ssh://git@github.com/OnixResearch/valence.git", rev = "7546a78b4eeef8686c6837e187aa12420f4f03bb", package = "valence-core" }
```

The selected revision contains `cairn/archive/2026-07-12-harden-preserves-integrity-boundaries`, including the corrected Preserves integrity and typed claim-boundary profile. The Nickel-authored cutover manifest at `standards/valence-core-cutover/manifest.ncl` binds that revision, Cargo source identity, archive path, module source hashes, fixture commands, and non-claims. Its generated runtime projection is `standards/valence-core-cutover/generated-manifest.json`.

The old Octet-hosted source remains only at `crates/valence-core/` as the non-publishable, non-workspace package `octet-valence-legacy-fixture`. It is a bounded positive/negative compatibility corpus and cannot satisfy a canonical `valence-core` dependency.

The selected module is the independently reviewed upstream `function_object.rs`
at BLAKE3 `11f0f4233aaf3153966f6aa37de570c2dcd0e75335591934461ae2b25ab3ecb1`.
The historical `source-token-v2/candidate-source-manifest.json` remains
`PENDING`, including its earlier consumer-test hash. The separate
`source-token-v2/reviewed-source-manifest.json` binds its independently audited
candidate snapshot, exact 37 source-byte hashes, 18-path internal/36-path
expanded selections, four saved receipt/taxonomy references, and immutable
upstream revision/module. Owner H2 authorization and independent peer audit
do not imply a cryptographic signature or owner review of later consumer-test
code. The existing named object Nix rails now require a fail-closed admission
of the frozen candidate/reviewed manifest, exact selected source and reference
bytes, scoped approval evidence, and the Cargo/Nix Valence pin before running
source-backed v2 tests. The fresh-build rail compares two independent binaries
against both saved internal and expanded receipt bytes. Report a rail or the
aggregate as PASS only after its derivation actually builds. A local Cargo
`[patch]` is only a disposable diagnostic, never this selected dependency.
The verifier and its expected manifest digests live in the same PR-controlled
tree as the rails, so an in-repository PASS does not itself authenticate H2
against adversarial edits: an out-of-PR reviewer must compare externally
frozen candidate BLAKE3 `6d79a8b09c4e7dd20236f4a141c01d6c18b793df7aad93e8ee4ec1d7fb83b221`
and reviewed BLAKE3 `2f420d0464e67a8b6ae518e7215eefd1ba8afc47d1849c07afedf284f196da29`.
The compiled producer's `valence_core_shim.rs` and `object_core.rs` are not
among the three H2-frozen Octet modules; the approved boundary covers
selected sources/references and observed producer scenarios, not unreviewed
future changes to every compiled producer module.
The shared scanner was corrected to balance
outer-attribute brackets through literal/comment content and capture split
visibility/modifiers. A rustc ABI repro showed that distinct multiline
export-name attributes previously yielded one object hash; `pub` on its own
line also previously collided with a private function of the same body.
Lexical marker association now refuses a matching reviewed `// octet-object:`
inside a block comment and marker-like fingerprint comments/attributes inside
non-code; a real marker immediately before outer attributes still attaches.
The current caller requests the v2 label through Valence's typed constructor;
the selected pin emits a source-backed v2 object. The historical v1 corpus
receipt stays readable but reports an `object_set_hash` mismatch under default
v2 source replay. Neither a passing self-produced v2 replay nor a local
diagnostic clears the existing Nix rails without independently bound sources
and saved references.

## Consumer migration

Downstream consumers such as Aspen must replace the Octet-hosted package with the exact standalone dependency above. A local `../valence/crates/valence-core` path may be used only as an explicit development diagnostic; it is not release evidence and must not change the manifest or lock identity.

After changing the dependency, run:

```bash
nix develop -c cargo test -p cargo-octet valence_cutover -- --nocapture
nix develop -c cargo test -p cargo-octet evidence_index -- --nocapture
nix develop -c cargo test -p cargo-octet object -- --nocapture
```

Passing these checks proves selected dependency/source ownership, unique canonical package identity, and compatibility for the declared Function Object, evidence-index, and evidence-codebase-index fixtures only. It does not prove behavioral correctness, arbitrary semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, release eligibility, or whole-program safety.
