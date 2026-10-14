## Contract

- [x] [serial] Define the initial evidence-tier enum and discriminants. r[octet.evidence_tiers.enum]
- [x] [serial] Define validation rules for `formal_proof` and `manual_review_required`. r[octet.evidence_tiers.tier_validation]
- [x] [serial] Register the evidence-tier artifact family in the policy registry with the correct ownership classification. r[octet.evidence_tiers.registry]

## Implementation

- [x] [serial] Add Rust-owned DTOs for evidence tiers in the relevant sidecar or receipt module. r[octet.evidence_tiers.enum]
- [x] [serial] Thread evidence-tier fields into sidecar findings or new evidence summaries without changing existing CLI exit-code semantics. r[octet.evidence_tiers.enum]
- [x] [serial] Generate or refresh Nickel contract artifacts and registry entries for emitted receipt families. r[octet.evidence_tiers.registry]
- [x] [serial] Document the tier meanings in consumer-facing evidence docs. r[octet.evidence_tiers.enum]

## Verification

- [x] [serial] Add positive serialization/contract fixtures for every initial evidence tier. r[octet.evidence_tiers.enum]
- [x] [serial] Add negative tests rejecting unknown tiers and `formal_proof` records with no proof artifact identity. r[octet.evidence_tiers.tier_validation]
- [x] [serial] Run targeted Rust tests for the DTO module and policy freshness checks. r[octet.evidence_tiers.registry]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.evidence_tiers.enum]
