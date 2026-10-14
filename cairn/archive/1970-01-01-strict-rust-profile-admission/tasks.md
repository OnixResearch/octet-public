## Phase 1: Policy and shared core

- [x] [serial] Add the typed Nickel strict-profile catalog, valid/invalid fixtures, checked-in generated runtime JSON, policy-family registry entry, and deterministic freshness check. r[octet.strict_rust_profiles.catalog] r[octet.strict_rust_profiles.catalog.valid] r[octet.strict_rust_profiles.catalog.invalid]
- [x] [serial] Add shared pure Rust DTO, catalog lookup, uniqueness, required-lint, mode, scope, and non-claim validation in `octet-lint-metadata`. r[octet.strict_rust_profiles.catalog] r[octet.strict_rust_profiles.catalog.invalid]

## Phase 2: Runner and driver enforcement

- [x] [serial] Add `cargo octet check --profile` parsing, known-profile resolution, supported-profile diagnostics, and scrubbed runner-to-driver selection handoff. r[octet.strict_rust_profiles.selection] r[octet.strict_rust_profiles.selection.known] r[octet.strict_rust_profiles.selection.unknown] r[octet.strict_rust_profiles.boundaries.default]
- [x] [serial] Add the pure workspace-profile preparation core plus thin `dylint.toml` shell for deterministic core-scope validation and path de-duplication. r[octet.strict_rust_profiles.core_scope] r[octet.strict_rust_profiles.core_scope.valid] r[octet.strict_rust_profiles.core_scope.missing]
- [x] [serial] Apply selected required lints at deny in the lint-driver config core while preserving unrelated consumer settings and existing explicit suppression governance. r[octet.strict_rust_profiles.lint_floor] r[octet.strict_rust_profiles.lint_floor.strengthened] r[octet.strict_rust_profiles.lint_floor.suppression]
- [x] [serial] Compute effective function-address mode and paths from profile floors without weakening stronger explicit operator configuration. r[octet.strict_rust_profiles.function_address_floor] r[octet.strict_rust_profiles.function_address_floor.raise] r[octet.strict_rust_profiles.function_address_floor.preserve]
- [x] [serial] Bind selected profile identity into BLAKE3 configuration/profile hashes, summary/status metadata, provenance checked scope, and existing artifact verification. r[octet.strict_rust_profiles.evidence] r[octet.strict_rust_profiles.evidence.visible] r[octet.strict_rust_profiles.evidence.identity]

## Phase 3: Documentation and verification

- [x] [serial] Document profile semantics, bootstrap commands, suppression behavior, compatibility defaults, and source-shape/proof/release non-claim boundaries in README/help and a focused operator guide. r[octet.strict_rust_profiles.boundaries] r[octet.strict_rust_profiles.boundaries.default] r[octet.strict_rust_profiles.boundaries.non_claims]
- [x] [serial] Add positive and negative Rust tests for catalog validation, CLI selection, lint-floor strengthening, core-scope preparation, function-address floors, default compatibility, and profile-bound evidence identity. r[octet.strict_rust_profiles.verification] r[octet.strict_rust_profiles.verification.focused]
- [x] [serial] Run focused Rust tests, Nickel fixture/freshness checks, a profile-bound artifact verification smoke, formatting, and Cairn validation/gates; record exact commands and outcomes in `cairn/changes/strict-rust-profile-admission/evidence/validation.md` before sync and archive. r[octet.strict_rust_profiles.verification.focused]
