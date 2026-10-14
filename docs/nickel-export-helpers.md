# Nickel export helpers

Octet uses Nickel at generation time for human-authored policy/config surfaces, then consumes deterministic native artifacts at runtime.

The export helper contract lives in `standards/nickel-exports/contracts.ncl`. A spec declares:

- `family_id`
- source Nickel file
- import dependencies
- optional selector and contract
- output format: `json`, `toml`, `yaml`, or `raw`
- output path

`octet-standards nickel-export-manifest` records source, dependency, output, command, byte length, and BLAKE3 identity after the shell has produced the output bytes. The manifest is evidence for freshness; it is not a runtime dependency for ordinary `cargo octet check`.

Secret-bearing exports are rejected by the Rust helper core unless the family is explicitly excluded or carries a non-candidate rationale. Tool-native files should stay native when Nickel composition adds no validation value.
