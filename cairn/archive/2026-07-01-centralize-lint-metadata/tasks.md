## Contract

- [x] [serial] Specify the Rust-owned lint catalog shape and required fields. r[octet.lint_metadata.single_source]
- [x] [serial] Specify runner metadata coverage for every registered lint. r[octet.lint_metadata.runner_coverage]
- [x] [serial] Specify docs and template freshness requirements for catalog-derived surfaces. r[octet.lint_metadata.docs_freshness]

## Implementation

- [x] [serial] Add the shared catalog source or generated metadata artifact and route lint registration diagnostics through it. r[octet.lint_metadata.single_source]
- [x] [serial] Replace or derive `cargo-octet` output metadata from the catalog. r[octet.lint_metadata.runner_coverage]
- [x] [serial] Refresh README, lint reference, templates, and fixture snapshots from the catalog. r[octet.lint_metadata.docs_freshness]

## Verification

- [x] [serial] Add positive tests proving every registered lint has catalog, docs, and runner metadata. r[octet.lint_metadata.single_source]
- [x] [serial] Add negative drift fixtures or tests for missing runner metadata and missing docs entries. r[octet.lint_metadata.runner_coverage]
- [x] [serial] Run `nix develop -c cargo test --lib`, `nix develop -c cargo test -p cargo-octet output`, and Cairn validation/gates. r[octet.lint_metadata.docs_freshness]
