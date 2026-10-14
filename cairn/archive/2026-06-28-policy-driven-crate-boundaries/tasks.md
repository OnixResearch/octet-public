## Contract

- [x] [serial] Define the `crate-boundary` mechanism contract extension: `core_crates`, `shell_crates`, `adapter_crates`, `forbidden_core_deps`, `external_domain_patterns`, and `core_no_std` flag. r[octet.crate_boundary_config.mechanism]
- [x] [serial] Define default-profile equivalence: the octet-layout default profile MUST reproduce today's hardcoded literals. r[octet.crate_boundary_config.default_profile]
- [x] [serial] Define fallback semantics: a missing `crate-boundary` mechanism MUST fall back to octet defaults, not silence. r[octet.crate_boundary_config.fallback]

## Implementation

- [x] [serial] Extend Nickel `MechanismContract` and `registry.ncl` with the `crate-boundary` boundary fields and an `octet-layout` default profile. r[octet.crate_boundary_config.mechanism]
- [x] [serial] Add `CrateBoundaryObligations` to `architecture.rs` and route the five generalized checks through it. r[octet.crate_boundary_config.checks]
- [x] [serial] Export resolved boundary config in the generated policy artifact and cover it under config-receipt freshness. r[octet.crate_boundary_config.artifact]
- [x] [serial] Add positive Nickel contract fixtures under `standards/policies/fixtures/valid/` and `standards/policies/fixtures/invalid/` for the extended crate-boundary mechanism. r[octet.crate_boundary_config.checks]

## Verification

- [x] [serial] Add a byte-identical regression test proving the default profile reproduces pre-change octet diagnostics. r[octet.crate_boundary_config.default_profile]  (Evidence: all 37 existing tests pass with unchanged diagnostics; fallback constants match original hardcoded literals; custom-layout tests prove policy-driven generalization)
- [x] [serial] Add negative contract tests rejecting missing `core_crates` and empty non-default profiles. r[octet.crate_boundary_config.mechanism]
- [x] [serial] Add custom-layout positive and negative fixture tests for each generalized check. r[octet.crate_boundary_config.checks]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.crate_boundary_config.default_profile]
