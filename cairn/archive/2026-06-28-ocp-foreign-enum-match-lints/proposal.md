## Why

Octet's only OCP proxy is `catch_all_on_enum`, which flags `_` match arms hiding new variants. It is narrow: it does not distinguish a `match` on an enum defined in *another crate* (closed-for-extension smell) from a `match` on your own enum, and it has no inverse — nothing flags an exhaustive match on your own enum that will silently break when you add a variant (open-for-extension-but-fragile). Both are OCP-relevant signals that the compiler cannot give without intent declarations.

These are advisory Dylint lints that fit `src/safety/` next to `catch_all_on_enum` and follow the standard lint-adding checklist.

## What Changes

- Add `foreign_enum_catch_all` (safety, `allow`): flag `match` on an enum from another crate that uses a `_` arm, since the foreign enum can grow without notice.
- Add `fragile_exhaustive_enum_match` (safety, `allow`): flag exhaustive `match` on a same-crate enum without a `_` arm, marking it fragile to future variant additions. Optionally honor a marker attribute (e.g. `#[octet::sealed_enum]` or an opt-in) to mark enums deliberately closed.
- Register both lints, add diagnostic metadata, add config keys for the opt-in marker, and add UI fixtures with `.stderr` snapshots.

## Impact

- **Files**: new files under `src/safety/` (`foreign_enum_catch_all.rs`, `fragile_exhaustive_enum_match.rs`), `src/lib.rs`, `src/diagnostics.rs`, `src/config.rs`, `src/safety/mod.rs`, `docs/lint-reference.md`, `templates/dylint.toml`, `tests/fixtures/ui_fixture_set.json`, `tests/fixtures/lint_emitting_fixtures.json`, new `ui_tests/` fixtures.
- **Testing**: positive and negative UI fixtures per lint; cross-crate fixture proving `foreign_enum_catch_all` only fires on foreign enums; the opt-in marker fixture proving `fragile_exhaustive_enum_match` is suppressed for sealed enums; `BLESS=1` snapshot refresh; `cargo test --lib`; `autofix_machine_applicable` rail; `cairn validate --root .` and Cairn gates.
