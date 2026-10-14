## Contract

- [x] [serial] Define the ProofEvidenceRef DTO fields and non-trait boundary. r[octet.verus_proof_rails.proof_evidence_ref]
- [x] [serial] Define read-only CLI validation against `proof-cards.json`. r[octet.verus_proof_rails.proof_evidence_ref.cli]
- [x] [serial] Define exact linkage diagnostics for ID, scope, evidence tier, status, artifacts, and non-claims. r[octet.verus_proof_rails.proof_evidence_ref.diagnostics]

## Implementation

- [x] [serial] Add the pure ProofEvidenceRef validation core and serde DTOs. r[octet.verus_proof_rails.proof_evidence_ref]
- [x] [serial] Add `cargo octet proof-ref validate --proof-cards PATH --ref PATH [--output-format human|json]`. r[octet.verus_proof_rails.proof_evidence_ref.cli]
- [x] [serial] Add help text and parser routing for the new command. r[octet.verus_proof_rails.proof_evidence_ref.cli]
- [x] [serial] Add positive and negative unit tests for proof-ref validation. r[octet.verus_proof_rails.proof_evidence_ref.diagnostics]

## Verification

- [x] [serial] Run `cargo test -p cargo-octet proof_ref` inside the nix devshell. r[octet.verus_proof_rails.proof_evidence_ref]
- [x] [serial] Run targeted parser/help tests if they are not covered by the proof-ref filter. r[octet.verus_proof_rails.proof_evidence_ref.cli]
- [x] [serial] Run `nix run .#cairn -- validate --root .` and Cairn gates for this change. r[octet.verus_proof_rails.proof_evidence_ref]
