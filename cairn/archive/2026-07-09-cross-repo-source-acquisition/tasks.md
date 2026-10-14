## Tasks

- [x] [serial] Define source-acquisition receipt DTOs and Nickel contracts. r[octet.dependency_trust.source_acquisition_receipts]
- [x] [serial] Add positive fixtures for registry, public Git, approved private SSH, vendored snapshot, Nix flake, and local override sources. r[octet.dependency_trust.source_acquisition_receipts.positive]
- [x] [serial] Add negative fixtures for unpinned Git, path-only release source, missing credential boundary, stale lock identity, and weakened non-claims. r[octet.dependency_trust.source_acquisition_receipts.negative]
- [x] [serial] Implement pure source-acquisition validation and shell metadata extraction for Cargo/Nix inputs. r[octet.dependency_trust.source_acquisition_receipts.validation]
- [x] [serial] Integrate source-acquisition receipts with artifact verification and dependency-trust summaries without merging evidence tiers. r[octet.dependency_trust.source_acquisition_receipts.evidence_separation]
- [x] [serial] Document consumer guidance for stack-owned private SSH and local sibling overrides. r[octet.dependency_trust.source_acquisition_receipts.docs]
- [x] [serial] Run focused tests, contract checks, artifact verification, and Cairn validation/gates for this change. r[octet.dependency_trust.source_acquisition_receipts.final_validation]
