## Why

Octet currently describes, registers, and emits lint metadata through several independent surfaces. The audit found concrete drift: the lint inventory in `src/lib.rs` registers more lints than the README and lint reference claim, `cargo-octet/src/output.rs` lacks metadata for some registered lints, and docs/templates describe `octet::...` attrs while live registered groups still expose `tigerstyle::...` in places.

A single source of truth prevents silent catalog drift and keeps runner sidecars, SARIF, docs, fixture snapshots, and rollout templates aligned with the actual lint inventory.

## What Changes

- Introduce a Rust-owned lint metadata source that covers every registered lint, domain group, default level, guidance text, effect family, documentation summary, and canonical namespace.
- Generate or validate consumer-facing docs and runner output metadata from that source.
- Add drift tests that fail when a lint is registered without matching metadata, docs, and runner output coverage.
- Update README and lint-reference counts from generated or checked metadata instead of hand-maintained values.

## Impact

- **Files**: `src/lib.rs`, `src/diagnostics.rs`, `cargo-octet/src/output.rs`, `docs/lint-reference.md`, `README.md`, `templates/dylint.toml`, `tests/fixtures/*`, and possibly a small shared metadata crate or generated metadata artifact.
- **Testing**: metadata coverage tests, docs freshness tests, runner output metadata tests, and focused cargo-octet JSON/SARIF fixture checks.
