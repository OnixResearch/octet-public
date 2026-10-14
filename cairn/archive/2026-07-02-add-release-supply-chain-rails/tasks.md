## Contract

- [x] [serial] Define auditable-binary release rail behavior and missing-metadata failure contract. r[octet.release_supply_chain.auditable_binary]
- [x] [serial] Define SBOM generation, vulnerability scan receipt, finding, and parent-link behavior. r[octet.release_supply_chain.sbom_scan]
- [x] [serial] Define attestation-shaped local receipt fields and hosted-service non-requirement. r[octet.release_supply_chain.attestation_compat]
- [x] [serial] Extend parent artifact chain wording for release supply-chain receipts. r[octet.evidence_provenance.parent_chain]

## Implementation

- [x] [serial] Add pure release supply-chain receipt validation and parent-link checks. r[octet.release_supply_chain.attestation_compat]
- [x] [serial] Add optional tool adapters for auditable binary checks, binary audit, SBOM generation, and vulnerability scans. r[octet.release_supply_chain.auditable_binary]
- [x] [serial] Add receipt/provenance DTOs for SBOM and scan outputs. r[octet.release_supply_chain.sbom_scan]
- [x] [serial] Add docs explaining local-first release evidence and SLSA/in-toto compatibility boundaries. r[octet.release_supply_chain.attestation_compat]

## Verification

- [x] [serial] Add positive auditable-binary metadata fixture and negative missing-metadata fixture. r[octet.release_supply_chain.auditable_binary]
- [x] [serial] Add SBOM receipt plus vulnerability-finding fixture with parent provenance. r[octet.release_supply_chain.sbom_scan]
- [x] [serial] Add artifact verification tests for missing parent, hash mismatch, and hosted-service non-requirement text. r[octet.release_supply_chain.attestation_compat]
- [x] [serial] Run targeted release rail tests, artifact verification tests, and Cairn validation/gates. r[octet.evidence_provenance.parent_chain]
