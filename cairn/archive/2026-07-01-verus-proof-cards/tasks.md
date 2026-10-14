## Contract

- [x] [serial] Specify required proof-card fields, receipt linkage, BLAKE3 artifact identity fields, and evidence-tier boundaries. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Specify fail-closed behavior for stale, incomplete, imported-as-formal-proof, or overclaiming proof cards. r[octet.verus_proof_rails.proof_cards]

## Implementation

- [x] [parallel] Add the proof-card review surface for the current Verus fixture rail, production pilot rails, and imported Trellis proof-bundle fixtures. r[octet.verus_proof_rails.proof_cards]
- [x] [parallel] Link cards from the Verus proof-path explainer or `verus/README.md` without replacing receipt/provenance review. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Ensure cards are generated from structured receipt/provenance data or checked against manifest/receipt fields before they can report `formal_proof`. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Ensure imported Trellis cards expose BLAKE3 identities while remaining external/manual-review evidence unless Octet reruns and validates the proof. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Preserve starter-path independence so ordinary `cargo octet check` does not require proof cards, imported bundles, or Verus artifacts. r[octet.verus_proof_rails.proof_cards]

## Verification

- [x] [serial] Add or run positive card inventory checks proving every current Verus rail card and imported Trellis card fixture contains the required fields. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Add or run positive receipt-linkage checks proving every card that reports `formal_proof` points to a matching validated receipt/provenance artifact. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Add or run positive imported-reference checks proving Trellis cards include repository revision and BLAKE3 source/proof/log/receipt identities while staying non-`formal_proof` unless Octet reruns the proof. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Add or run negative checks for missing required fields, stale receipt links, fixture evidence mislabeled as production, external evidence mislabeled as formal proof, and broad correctness overclaims. r[octet.verus_proof_rails.proof_cards]
- [x] [serial] Run Cairn validation and gates for this change package before implementation and before archive. r[octet.verus_proof_rails.proof_cards]
