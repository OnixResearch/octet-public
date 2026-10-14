## Contract

- [x] [serial] Specify the Trellis imported-proof bundle fields, evidence-origin values, and BLAKE3 identity rules. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Specify promotion rules from imported external reference to Octet `formal_proof` through rerun and validated Octet receipt only. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Specify registry ownership for Nickel-authored import manifest, generated JSON, and any Rust-derived import validation receipts. r[octet.verus_proof_rails.trellis_blake3_import]

## Implementation

- [x] [serial] Add a typed Nickel import manifest contract for selected Trellis proof references. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Add deterministic JSON export and freshness coverage for the import manifest. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Add pure import validation core for required fields, normalized paths, BLAKE3 digest shape, computed hash comparison, duplicate identity detection, evidence-tier rules, and trust-boundary notes. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Add a thin shell or artifact-verifier integration that reads imported artifacts, computes BLAKE3 hashes, and calls the pure validation core. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [parallel] Link imported Trellis bundles from proof cards and the proof-path explainer without replacing Octet receipts/provenance. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Update README references to record `../trellis/` as the reference proof corpus used by this change. r[octet.verus_proof_rails.trellis_blake3_import]

## Verification

- [x] [serial] Add positive import fixture validation for repository revision, normalized source/proof paths, hash algorithm, BLAKE3 source/proof/log/receipt identities, trust boundaries, and external/manual-review tier. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Add positive rerun-path fixture showing the imported bundle names the Octet proof command required for promotion without claiming promotion happened. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Add negative fixtures for missing required fields, malformed digest shape, stale source/proof/log hash, duplicate identity, unsupported hash algorithm, and imported-as-`formal_proof` without Octet receipt. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Run policy freshness, registry, focused import validation, proof-card, artifact-verifier, and starter-path independence checks after implementation. r[octet.verus_proof_rails.trellis_blake3_import]
- [x] [serial] Run Cairn validation and gates for this change package before implementation and before archive. r[octet.verus_proof_rails.trellis_blake3_import]
