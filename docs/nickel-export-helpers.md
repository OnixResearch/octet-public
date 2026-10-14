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

Secret-bearing authored source or dependency material is rejected by the standalone core unless the family explicitly permits it. Contract labels remain Octet-owned metadata; captured contract failures are supplied to the core as structured error diagnostics. Tool-native files should stay native when Nickel composition adds no validation value.

`standards/nickel-export-cutover/manifest.ncl` binds the Cargo dependency, flake input, lock identity, source hashes, compatibility disposition, and non-claims. Local path overrides are diagnostic-only and cannot replace the release source.

`standards/nickel-export-cutover/migration-profiles.ncl` records consumer-owned handoffs for Mantle, Cairn, Trellis, and Animus, including each local handoff commit, retained authority, dual-run matrix, and rollback boundary. Those commits establish reviewable local lifecycle packages only; publication and implementation remain owned by each consumer repository.
