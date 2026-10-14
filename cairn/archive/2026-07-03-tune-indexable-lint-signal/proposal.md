## Why

Octet's own indexable-policy dogfood run proved that several naming and indexing lints emit a large amount of accepted debt rather than clear action. The archived run recorded 653 findings, dominated by `non_trait_imports`, `path_segment_repetition`, `underscore_in_module_filename`, and `renamed_imports`. The triage accepted the findings because the current lint semantics are too broad for rustc-private lint implementation code and would cause broad mechanical churn.

Those lints still encode useful review principles, but they need scope-aware signal shaping before they can move from inventory to reliable rollout guidance.

## What Changes

- Add a signal-quality contract for high-volume naming/indexable lints.
- Add scoped exception and allowlist design for rustc-private lint implementation code, canonical aliases such as `rustc_hir as hir`, generated/searchability preserving names, and module layouts that intentionally mirror lint identities.
- Require dogfood evidence to report raw findings, accepted-debt counts, fixed counts, and residual noise by lint before any promotion.

## Impact

- **Files**: `src/naming/*`, `src/structure/*` for affected lints, `src/config.rs`, `crates/octet-lint-metadata`, `docs/downstream-rollout.md`, `templates/dylint.toml`, UI fixtures, and dogfood evidence helpers.
- **Testing**: positive and negative UI fixtures for each exception class, config parsing tests, focused dogfood rerun, and Cairn validation/gates.
