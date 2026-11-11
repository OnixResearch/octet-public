## Why

Tigerstyle now has typed Nickel policy contracts and structural enforcement metadata, but consumers still need a stable artifact seam between Nickel authoring and Rust checker execution. A direct `nickel export` command is useful for maintainers, while consumer gates need named generated files that CI can cache, diff, review, and feed back into `tigerstyle-standards` without reinterpreting Nickel.

## What Changes

- Add deterministic generated policy artifacts: `tigerstyle-policies.json`, `tigerstyle-check-config.toml`, and `tigerstyle-seed-manifests.json`.
- Add a standards CLI export path that writes those artifacts from the Nickel-exported registry JSON.
- Allow standards checks to consume the generated artifact directory directly.
- Extend the Nix fixture gate to prove artifact export and artifact-backed checking.

## Impact

- **Files**: `crates/tigerstyle-standards`, `flake.nix`, policy docs, and standards-codification spec.
- **APIs**: Adds `tigerstyle-standards export-artifacts` and `tigerstyle-standards check --policy-artifacts-dir`.
- **Testing**: Validate with focused CLI tests, package tests, policy validation, and the Nix standards-fixtures check.
