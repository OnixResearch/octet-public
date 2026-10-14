## Contract

- [x] [serial] Define the provenance record fields and BLAKE3 hash requirement. r[octet.evidence_provenance.record]
- [x] [serial] Define fail-closed validation for missing source IDs, unknown checker IDs, and hash mismatches. r[octet.evidence_provenance.fail_closed]
- [x] [serial] Register provenance records in the config/receipt family registry. r[octet.evidence_provenance.registry]

## Implementation

- [x] [serial] Add a Rust-owned provenance DTO and pure validation core. r[octet.evidence_provenance.record]
- [x] [serial] Wire the runner or standards CLI shell to compute BLAKE3 artifact hashes and serialize provenance records. r[octet.evidence_provenance.record]
- [x] [serial] Add registry coverage and generated contract freshness checks for the provenance record family. r[octet.evidence_provenance.registry]
- [x] [serial] Document how release operators trace a requirement or policy ID to an artifact. r[octet.evidence_provenance.record]

## Verification

- [x] [serial] Add a positive fixture for a valid source-to-artifact provenance record. r[octet.evidence_provenance.record]
- [x] [serial] Add negative fixtures for missing source ID, unknown checker ID, and hash mismatch. r[octet.evidence_provenance.fail_closed]
- [x] [serial] Run targeted Rust tests for provenance validation and policy freshness. r[octet.evidence_provenance.registry]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.evidence_provenance.record]
