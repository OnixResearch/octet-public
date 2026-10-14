## Contract

- [x] [serial] Specify Rust-owned generated Nickel contracts for emitted finding and provenance DTOs. r[octet.evidence_contracts.generated]
- [x] [serial] Specify freshness registration for generated evidence contracts. r[octet.evidence_contracts.registry]
- [x] [serial] Specify fail-closed contract fixtures for malformed evidence records. r[octet.evidence_contracts.fail_closed]

## Implementation

- [x] [serial] Add a deterministic Rust-owned schema source and generator for `Finding` v2 and `ProvenanceRecord`. r[octet.evidence_contracts.generated]
- [x] [serial] Commit generated Nickel contracts and wire them into policy validation freshness checks. r[octet.evidence_contracts.registry]
- [x] [serial] Update contributor and policy docs to state the Rust-derived ownership rule for emitted evidence DTOs. r[octet.evidence_contracts.generated]

## Verification

- [x] [serial] Add positive fixtures for valid finding and provenance records. r[octet.evidence_contracts.generated]
- [x] [serial] Add negative fixtures for missing required fields, invalid evidence tier, unsupported schema version, and wrong field types. r[octet.evidence_contracts.fail_closed]
- [x] [serial] Run contract freshness checks, policy validation, targeted cargo-octet tests, and Cairn validation/gates. r[octet.evidence_contracts.registry]
  - evidence: `nix develop -c cargo test -p cargo-octet evidence_contracts::tests::generated_contracts_are_fresh`
  - evidence: `nix develop -c cargo test -p cargo-octet evidence_contracts::tests::evidence_contract_fixtures_accept_valid_and_reject_invalid_records`
  - evidence: `nix develop -c cargo test -p cargo-octet evidence`
  - evidence: `nix develop -c nix run .#octet-policy-validate`
