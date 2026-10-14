## Contract

- [x] [serial] Define the `fat_trait` lint contract: flag traits exceeding `trait_method_max` (default 7), default `warn`. r[octet.solid_structure_lints.fat_trait]
- [x] [serial] Define the `concrete_construction_in_core` lint contract: flag external concrete construction in declared core scopes, default `warn`, no-op without a declared scope. r[octet.solid_structure_lints.concrete_construction_in_core]
- [x] [serial] Define the `type_accumulation` lint contract: flag excessive `impl` blocks or `pub` items per type/module, default `allow`. r[octet.solid_structure_lints.type_accumulation]
- [x] [serial] Define the `defaulting_trait_method` lint contract: flag trait methods with default bodies, default `allow`. r[octet.solid_structure_lints.defaulting_trait_method]

## Implementation

- [x] [serial] Implement `fat_trait` under `src/structure/`, register in `src/lib.rs`, add diagnostic metadata and the `trait_method_max` config key. r[octet.solid_structure_lints.fat_trait]
- [x] [serial] Implement `concrete_construction_in_core` under `src/structure/`, no-op without a declared core scope. r[octet.solid_structure_lints.concrete_construction_in_core]
- [x] [serial] Implement `type_accumulation` under `src/structure/` with `impl_blocks_max` and `module_pub_items_max` config keys. r[octet.solid_structure_lints.type_accumulation]
- [x] [serial] Implement `defaulting_trait_method` under `src/safety/`. r[octet.solid_structure_lints.defaulting_trait_method]
- [x] [serial] Update `README.md`, `docs/lint-reference.md`, `templates/dylint.toml` catalog summary and domain counts (38 -> 42). r[octet.solid_structure_lints.fat_trait]

## Verification

- [x] [serial] Add positive and negative UI fixtures for `fat_trait` and refresh `.stderr` snapshots. r[octet.solid_structure_lints.fat_trait]
- [x] [serial] Add positive and negative UI fixtures for `concrete_construction_in_core`, including a no-op fixture with no declared core scope. r[octet.solid_structure_lints.concrete_construction_in_core]
- [x] [serial] Add positive and negative UI fixtures for `type_accumulation`. r[octet.solid_structure_lints.type_accumulation]
- [x] [serial] Add positive and negative UI fixtures for `defaulting_trait_method`. r[octet.solid_structure_lints.defaulting_trait_method]
- [x] [serial] Update `tests/fixtures/ui_fixture_set.json` and `tests/fixtures/lint_emitting_fixtures.json` for all four lints. r[octet.solid_structure_lints.fat_trait]
- [x] [serial] Run `nix develop -c bash -c 'cargo test --lib'`, the UI rail, the autofix rail, and Cairn gates. r[octet.solid_structure_lints.fat_trait]
