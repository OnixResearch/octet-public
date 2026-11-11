# Third-party notices and provenance

Octet's `AGPL-3.0-or-later` grant does not replace third-party licenses,
copyrights, or attribution. Preserve upstream notices when redistributing
source or binaries. This inventory covers identified checked-in material;
it is not an exhaustive license inventory of Cargo or Nix dependency closures.

## Dylint support sources

- Paths: `tests/support/dylint/driver/`, `tests/support/dylint/internal/`,
  including the internal template and `template.tar`.
- Upstream: <https://github.com/trailofbits/dylint>.
- Reviewed baseline: `0e0a71eefe6f01563d11acc6e1d7af1d505934a9`.
- Copyright (c) 2021-2022 Trail of Bits. Upstream package metadata also
  credits Samuel E. Moelius III; those author fields are retained.
- Upstream offers `MIT OR Apache-2.0`. The bundled copy is redistributed
  using the MIT option. Full texts: [MIT](LICENSES/dylint-MIT.txt) and
  [Apache-2.0](LICENSES/dylint-APACHE.txt).

Comparison with that immutable baseline found the internal subtree identical.
The driver differs in `Cargo.toml` and `build/main.rs`; these are Octet's local
support patches, not an unmodified upstream release. The production flake
separately fetches the pinned upstream source and adds a binary entry point.

### Copied Clippy helper

`tests/support/dylint/internal/src/match_def_path.rs` retains its source link
to Rust Clippy revision `f62f26965817f2573c2649288faa489a03ed1665`:

<https://github.com/rust-lang/rust-clippy/tree/f62f26965817f2573c2649288faa489a03ed1665>

Copyright (c) 2014-2025 The Rust Project Developers. The copied helper is
redistributed under the [MIT license](LICENSES/clippy-MIT.txt).

## Imported standards prose: publication blocked pending permission

`docs/standards/source/` includes material credited to Adam Murray in
`front-matter.adoc` and `tigerstyle-standards.adoc`. The archived
`openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/` records
migration from a standalone repository, but does not identify an immutable
upstream source, original license, or redistribution permission.

The attribution is preserved. Octet's project notice does not establish a
right to relicense this material. Before public distribution, maintainers
must recover and record the original source and license or written permission.

## Cairn policy provenance

The following files match Cairn revision
`695124d459574ba7aeba6097310d237f393c243c` byte for byte:

- `cairn-policy/upstream-default.ncl` (upstream `cairn-policy/default.ncl`)
- `cairn-policy/contracts.ncl`
- `cairn-policy/lifecycle-store-contracts.ncl`
- `cairn-policy/nominal-evidence-identities.ncl`

Upstream: <https://github.com/OnixResearch/cairn/tree/695124d459574ba7aeba6097310d237f393c243c>.
That revision's `docs/licensing.md` places repository lifecycle material not
carrying another notice under `AGPL-3.0-or-later`; the full text is in Octet's
[LICENSE](LICENSE). This records the observed source and grant, not a claim
that the pinned repository is currently accessible anonymously.

## Fetched dependencies and references

Fetched dependencies retain their own licenses. For each binary release,
inventory the exact locked Rust and native/Nix dependencies and include their
required notices and corresponding source. A reference link or proof citation
alone is not a claim that its implementation is bundled or relicensed.

The frozen `crates/valence-core/` compatibility fixture retains its existing
AGPL declaration. It is not the standalone Valence provider and does not
establish that provider's license. See its README and the release checklist.
