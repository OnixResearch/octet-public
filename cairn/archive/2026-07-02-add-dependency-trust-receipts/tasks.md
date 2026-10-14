## Contract

- [x] [serial] Define cargo-vet execution/import modes and dependency-trust receipt fields. r[octet.dependency_trust.vet_receipts]
- [x] [serial] Define exemption, deferred-audit, and import identity visibility requirements. r[octet.dependency_trust.exceptions]
- [x] [serial] Define dependency-trust non-claims and separation from dependency-audit evidence. r[octet.dependency_trust.non_claims]

## Implementation

- [x] [serial] Add pure dependency-trust receipt validation and outcome classification. r[octet.dependency_trust.vet_receipts]
- [x] [serial] Add runner shell support for `cargo vet` execution or saved-output import. r[octet.dependency_trust.vet_receipts]
- [x] [serial] Add receipt/provenance DTOs that preserve exemptions, gaps, imports, and review actions. r[octet.dependency_trust.exceptions]
- [x] [serial] Document cargo-vet as optional trust evidence separate from cargo-deny. r[octet.dependency_trust.non_claims]

## Verification

- [x] [serial] Add positive fully-covered dependency-trust fixture. r[octet.dependency_trust.vet_receipts]
- [x] [serial] Add negative unaudited-gap, missing-import-identity, and exemption-visibility fixtures. r[octet.dependency_trust.exceptions]
- [x] [serial] Add evidence summary tests proving dependency trust and dependency audit remain separate entries. r[octet.dependency_trust.non_claims]
- [x] [serial] Run targeted runner tests, artifact verification tests, and Cairn validation/gates. r[octet.dependency_trust.vet_receipts]
