## Why

Octet's structure domain covers functions and files but has gaps in the SOLID directions that HIR-level lints can catch locally, without the full standards-checker machinery. Four checks are genuinely missing and each maps to one SOLID principle:

- **ISP — `fat_trait`:** `too_many_parameters` bounds functions, but nothing bounds a `trait` whose implementors only use a subset. Fat interfaces violate ISP.
- **DIP — `concrete_construction_in_core`:** the static cousin of the ambient-authority check. Today `explicit_defaults` only catches `Default::default()`; nothing flags `Foo::new()` / `Foo::default()` for non-domain infrastructure types inside declared core scopes.
- **SRP — `type_accumulation`:** `excessive_file_length` and `module_file_count` proxy SRP, but nothing flags a single type accumulating many unrelated `impl` blocks or a module exporting many unrelated `pub` items.
- **LSP — `defaulting_trait_method` (advisory):** nothing flags trait methods with default impls that silently change contracts for implementors that rely on the default.

These are HIR-level Dylint lints that fit the existing `src/structure/` and `src/safety/` domains and the standard lint-adding checklist, independent of the standards-checker changes.

## What Changes

- Add `fat_trait` (structure, `warn`): flag traits exceeding a configurable method count (`trait_method_max`, default 7).
- Add `concrete_construction_in_core` (structure, `warn`): flag `Foo::new()` / `Foo::default()` for non-domain, non-`std` types inside declared core scopes.
- Add `type_accumulation` (structure, `allow`): flag a single type with N unrelated `impl` blocks or a module exporting many unrelated `pub` items.
- Add `defaulting_trait_method` (safety, `allow`): flag trait methods with default impls that silently alter contracts; advisory given high false-positive risk.
- Register each lint in `src/lib.rs`, add diagnostic metadata in `src/diagnostics.rs`, add config keys in `src/config.rs`, and add UI fixtures under `ui_tests/` with `.stderr` snapshots.

## Impact

- **Files**: new files in `src/structure/` (`fat_trait.rs`, `concrete_construction_in_core.rs`, `type_accumulation.rs`) and `src/safety/` (`defaulting_trait_method.rs`), `src/lib.rs`, `src/diagnostics.rs`, `src/config.rs`, `src/structure/mod.rs`, `src/safety/mod.rs`, `docs/lint-reference.md`, `templates/dylint.toml`, `tests/fixtures/ui_fixture_set.json`, `tests/fixtures/lint_emitting_fixtures.json`, new `ui_tests/` fixtures.
- **Testing**: positive and negative UI fixtures per lint (the repo requires both); `BLESS=1` snapshot refresh; `cargo test --lib`; the `autofix_machine_applicable` rail if any lint emits a machine-applicable fix. The `concrete_construction_in_core` lint needs a declared core scope, so it composes with `policy-driven-crate-boundaries` for real consumers but degrades to a no-op without one.
