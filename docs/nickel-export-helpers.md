# Nickel export helpers

Octet uses Nickel at generation time for human-authored policy/config surfaces, then consumes deterministic native artifacts at runtime. Evaluator-neutral admission, exact-byte identity, diagnostics, and receipt semantics come from `nickel-export-core` pinned to `OnixResearch/nickel-export` revision `257fafc1c746f1faf156207043a4c826bfb16d49`. Octet does not depend on the standalone external-evaluator CLI.

Octet retains its legacy spec and manifest projection in `crates/octet-standards/src/nickel_export.rs`; this facade delegates canonical admission and identity to the standalone core. The Octet-owned spec contract lives in `standards/nickel-exports/contracts.ncl`. A spec declares:

- `family_id`
- source Nickel file
- import dependencies
- optional selector and contract
- output format: `json`, `toml`, `yaml`, or `raw`
- output path

`octet-standards nickel-export-manifest` records source, dependency, output, command, byte length, and BLAKE3 identity after the shell has produced the output bytes. The manifest is evidence for freshness; it is not a runtime dependency for ordinary `cargo octet check`, and ordinary checks do not invoke Nickel or `nickel-export`.

The facade reports the dependency closure to the core as evaluator-observed. It builds that closure by scanning `import` targets in the source and, recursively, in the supplied Nickel dependency texts. It skips comments and string contents but scans string interpolations. Quoted targets resolve against the importing file's directory, and a bare package import keeps its name. The observed closure must equal the declared `dependencies` exactly. An undeclared import, including a transitive one, fails admission, as does a declared dependency that nothing imports. Every import counts as a dependency, but the scanner reads a dependency's text only when Nickel parses it as Nickel source. That is the case for an import tagged `as 'Nickel`, and for an untagged import whose extension is not `.json`, `.yaml`, `.yml`, `.toml`, or `.txt`. A dependency imported with another format tag, such as `as 'Text`, or untagged with one of those data extensions, is not scanned.

Secret-bearing authored source or dependency material is rejected by the standalone core unless the family explicitly permits it. Contract labels remain Octet-owned metadata; captured contract failures are supplied to the core as structured error diagnostics. Tool-native files should stay native when Nickel composition adds no validation value.

`standards/nickel-export-cutover/manifest.ncl` binds the Cargo dependency, flake input, lock identity, source hashes, compatibility disposition, and non-claims. Local path overrides are diagnostic-only and cannot replace the release source.

`standards/nickel-export-cutover/migration-profiles.ncl` records consumer-owned handoffs for Mantle, Cairn, Trellis, and Animus, including each local handoff commit, retained authority, dual-run matrix, and rollback boundary. Those commits establish reviewable local lifecycle packages only; publication and implementation remain owned by each consumer repository.
