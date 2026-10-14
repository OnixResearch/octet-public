## Why

`nix develop -c cargo check` currently succeeds but Cargo warns that the same `src/main.rs` file is present in multiple binary targets for `cargo-octet` aliases and `octet-standards` aliases. Octet already treats warning-clean checks as release hygiene, so routine success output should not include avoidable Cargo target warnings.

The compatibility commands still matter, but they should be declared in a way that keeps the canonical and legacy binaries warning-clean.

## What Changes

- Replace duplicate `path = "src/main.rs"` alias target declarations with a warning-clean alias-bin structure.
- Preserve canonical and compatibility command behavior for `cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`, `octet-standards`, `slotcar-standards`, and `tigerstyle-standards`.
- Add a focused regression check that fails if routine workspace checks reintroduce Cargo duplicate-target warnings.

## Impact

- **Files**: `cargo-octet/Cargo.toml`, `cargo-octet/src/`, `crates/octet-standards/Cargo.toml`, `crates/octet-standards/src/`, release smoke checks, and command alias tests.
- **Testing**: alias version/help tests, targeted cargo checks that assert no duplicate-target warning text, existing runner and standards tests, and build-hygiene validation.
