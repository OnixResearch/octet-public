# Indexable lint signal-quality summary

Source baseline: `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/dogfood-summary.txt`.

command: `python3 openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/run-indexable-dogfood.py`
enabled_lint_set: `drop_must_use`, `fallible_int_fallback`, `magic_number`, `excessive_file_length`, `module_file_count`, `underscore_in_module_filename`, `non_trait_imports`, `renamed_imports`, `path_segment_repetition`
disabled_lints_count: 38
raw_inventory_artifact: `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/dogfood-findings.jsonl`
configured_exception_classes: `rustc_private_aliases`, `lint_identity_names`, `generated_searchable_names`

This update adds scoped classifier support and UI guardrails for the highest-noise
naming/indexing classes, but it does not promote these lints. A full dogfood
rerun with the new classifiers must replace the residual counts below before any
starter or default-level promotion.

| Lint | Raw findings | Configured exceptions counted in latest full dogfood | Accepted debt | Fixed findings | Residual findings | Status |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `non_trait_imports` | 471 | 0 | 471 | 0 | 471 | blocked pending rerun with `rustc_private_aliases` summary |
| `path_segment_repetition` | 91 | 0 | 91 | 0 | 91 | blocked pending rerun with `lint_identity_names` summary |
| `renamed_imports` | 35 | 0 | 35 | 0 | 35 | blocked pending rerun with `rustc_private_aliases` summary |
| `underscore_in_module_filename` | 48 | 0 | 48 | 0 | 48 | blocked pending rerun with `lint_identity_names` summary |

Focused guardrail evidence in this change:

| Lint | Exception class | Positive guardrail | Negative guardrail |
| --- | --- | --- | --- |
| `non_trait_imports` | `rustc_private_aliases` | `ui_tests/non_trait_imports.rs` keeps `crate::rustc_hir as hir` clean | `use crate::addr::Addr` still warns |
| `renamed_imports` | `rustc_private_aliases` | `ui_tests/renamed_imports.rs` keeps `crate::rustc_hir as hir` clean | `Config as TransportConfig` still warns |
| `path_segment_repetition` | `lint_identity_names` | `ui_tests/path_segment_repetition.rs` keeps `path_segment_repetition_classifier` clean in the same lint-identity module | `guest::spawn_guest_vm` still warns |
| `underscore_in_module_filename` | `lint_identity_names` | `policy_inputs/underscore_module_filename/path_segment_repetition.rs` is included and clean | `compute_vm.rs` still warns |

Promotion decision: `blocked`. The scoped exceptions are safe to ship as signal
shaping, but these high-volume lints remain inventory-only until a saved dogfood
rerun reports non-zero exception counts, updated residual counts, and a
`ready`, `deferred`, or `blocked` decision for each lint.
