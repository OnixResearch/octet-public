## Phase 1: Canonical integrity core

- [x] [serial] Implement pure parsed Preserves self-hash field removal and canonical packed encoding. r[valence.evidence_ir.preserves_integrity.canonical_hash]
- [x] [serial] Introduce the corrected hash-profile identity and fail-closed profile selection. r[valence.evidence_ir.preserves_integrity.profile_version]
- [x] [parallel] Add positive canonicalization fixtures for stable encoding and legitimate payload occurrences of self-hash field names. r[valence.evidence_ir.preserves_integrity.fixtures.positive]
- [x] [parallel] Add negative collision fixtures proving altered payload content cannot retain the benign digest. r[valence.evidence_ir.preserves_integrity.fixtures.negative]

## Phase 2: Typed claim boundary

- [x] [serial] Add typed supported-claim, prohibited-capability, verification-role, and required-boundary contracts. r[valence.evidence_ir.typed_claim_boundary.contract]
- [x] [serial] Replace prose authority checks with pure typed policy evaluation while retaining prose as display metadata. r[valence.evidence_ir.typed_claim_boundary.validation]
- [x] [parallel] Add positive fixtures for bounded identity/linkage claims. r[valence.evidence_ir.typed_claim_boundary.fixtures.positive]
- [x] [parallel] Add negative fixtures for contradictory prose, unknown claim classes, role promotion, equivalence claims, authority claims, and release-eligibility claims. r[valence.evidence_ir.typed_claim_boundary.fixtures.negative]

## Phase 3: Migration and downstream contract

- [x] [serial] Add an explicit byte-bound legacy-to-corrected profile migration receipt without in-place receipt rewriting. r[valence.evidence_ir.preserves_integrity.migration]
- [x] [parallel] Document profile invalidation, migration ordering, downstream acceptance, and bounded claims. r[valence.evidence_ir.preserves_integrity.docs]

## Phase 4: Verification

- [x] [serial] Run focused positive and negative Rust and fixture tests for canonical hashing, migration, and typed claims. r[valence.evidence_ir.preserves_integrity.final_validation]
- [x] [serial] Add a checked-in CI workflow that runs only `nix flake check`. r[valence.evidence_ir.preserves_integrity.flake_check_ci]
- [x] [serial] Run `nix flake check` and Cairn validation/gates before sync and archive. r[valence.evidence_ir.preserves_integrity.final_validation]
