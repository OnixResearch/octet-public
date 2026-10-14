## Contract

- [x] [serial] Define the cargo-deny dependency-audit rail command/options and starter-path boundary. r[octet.dependency_audit.cargo_deny_rail]
- [x] [serial] Define config discovery, missing-config diagnostics, and locked advisory-database update behavior. r[octet.dependency_audit.config_boundary]
- [x] [serial] Define dependency-audit receipt fields and provenance mapping. r[octet.dependency_audit.receipt]

## Implementation

- [x] [serial] Add pure dependency-audit mode/config validation and receipt DTOs. r[octet.dependency_audit.receipt]
- [x] [serial] Add the runner shell that invokes `cargo deny check` with explicit config, workspace scope, and locked/update mode. r[octet.dependency_audit.cargo_deny_rail]
- [x] [serial] Add docs and a copyable `deny.toml` template or documented template path. r[octet.dependency_audit.config_boundary]
- [x] [serial] Register dependency-audit receipt/config families in policy and artifact freshness registries. r[octet.dependency_audit.receipt]

## Verification

- [x] [serial] Add a positive clean dependency-policy fixture. r[octet.dependency_audit.cargo_deny_rail]
- [x] [serial] Add negative fixtures for denied dependency policy and missing config. r[octet.dependency_audit.cargo_deny_rail]
- [x] [serial] Add receipt/provenance verification for dependency-audit artifacts. r[octet.dependency_audit.receipt]
- [x] [serial] Run targeted runner tests, policy freshness checks, and Cairn validation/gates. r[octet.dependency_audit.receipt]
