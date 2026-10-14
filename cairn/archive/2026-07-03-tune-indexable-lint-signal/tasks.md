## Contract

- [x] [serial] Specify the dogfood signal-quality budget and summary fields for high-volume lint families. r[octet.lint_signal_quality.dogfood_budget]
- [x] [serial] Specify scoped exception classes for rustc-private lint code, canonical aliases, generated/searchable names, and lint-identity-preserving module paths. r[octet.lint_signal_quality.scoped_exceptions]
- [x] [parallel] Specify promotion blockers when accepted-debt counts remain high. r[octet.lint_signal_quality.promotion_blockers]

## Implementation

- [x] [serial] Add pure classifier/config support for scoped exceptions in `non_trait_imports`, `renamed_imports`, `path_segment_repetition`, and `underscore_in_module_filename`. r[octet.lint_signal_quality.scoped_exceptions]
- [x] [serial] Update `templates/dylint.toml` and downstream rollout docs with cleanup-note requirements for any new exception controls. r[octet.lint_signal_quality.scoped_exceptions]
- [x] [parallel] Add or update dogfood helper output so raw, excepted, accepted, fixed, and residual counts are reported per lint. r[octet.lint_signal_quality.dogfood_budget]

## Verification

- [x] [serial] Add positive and negative UI fixtures for each exception class, including ordinary consumer cases that still warn. r[octet.lint_signal_quality.scoped_exceptions]
- [x] [serial] Save a focused dogfood summary proving residual noise changed or documenting why a lint remains inventory-only. r[octet.lint_signal_quality.dogfood_budget]
- [x] [serial] Run `nix develop -c bash -c 'cargo test --lib'`, the UI rail, and Cairn validation/gates. r[octet.lint_signal_quality.promotion_blockers]
