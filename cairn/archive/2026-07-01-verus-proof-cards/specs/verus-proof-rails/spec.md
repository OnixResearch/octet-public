## ADDED Requirements

### Requirement: Verus proof cards

r[octet.verus_proof_rails.proof_cards] Octet SHOULD provide receipt-backed proof cards for selected Verus proof rails and BLAKE3-identified imported proof references so reviewers can inspect each scoped proof claim without treating the card as a standalone proof artifact.

#### Scenario: Proof card contains required fields

r[octet.verus_proof_rails.proof_cards.fields]

- GIVEN Octet emits or documents a proof card for a Verus proof rail
- WHEN a reviewer inspects the card
- THEN the card SHOULD name the proof or property ID, source requirement or policy ID, checked Rust source scope, proof target path, verifier mode, verifier identity summary, solver/toolchain summary, receipt path, artifact BLAKE3 identities, evidence tier, status, trust-boundary notes, and non-claims.
- AND the card SHOULD link to the artifact-verifier command or receipt/provenance artifact that supports the status.

#### Scenario: Formal-proof card is receipt backed

r[octet.verus_proof_rails.proof_cards.receipt_backed]

- GIVEN a proof card reports `evidence_tier = formal_proof` or equivalent formal-proof status
- WHEN card validation runs
- THEN validation MUST confirm that the card is generated from, or matches, a validated structured proof receipt and matching provenance record for the same source scope, proof target, verifier mode, and artifact identity.
- AND a card without a matching validated receipt MUST NOT report `formal_proof` status.

#### Scenario: Fixture, production, and external modes stay distinct

r[octet.verus_proof_rails.proof_cards.mode_distinct]

- GIVEN proof cards exist for fixture-only, production, and imported external Verus evidence
- WHEN a reviewer compares the cards
- THEN each card MUST display its verifier mode and evidence origin.
- AND fixture-only cards MUST NOT be presented as production formal-proof evidence unless the checked scope explicitly declares fixture-only proof plumbing.
- AND imported Trellis cards MUST NOT be presented as Octet `formal_proof` evidence unless Octet reruns the proof with a pinned verifier and validates an Octet-owned receipt.

#### Scenario: Overclaiming card fails validation

r[octet.verus_proof_rails.proof_cards.no_overclaim]

- GIVEN a proof card mentions proof, safety, verification, correctness, certification, business-domain validation, or semantic equivalence
- WHEN card validation scans the claim language
- THEN the claim MUST be scoped to the declared property, source binding, proof binding, verifier identity, receipt, and artifact identity.
- AND cards MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime without separate evidence.

#### Scenario: Imported Trellis card preserves BLAKE3 scope

r[octet.verus_proof_rails.proof_cards.trellis_blake3]

- GIVEN a proof card references a Trellis proof bundle by BLAKE3 identity
- WHEN card validation runs
- THEN validation MUST require the Trellis repository revision, source artifact hash, proof artifact hash, bounded log hash, receipt hash when present, hash algorithm, and evidence-origin fields.
- AND the card MUST state that BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness.
- AND the card MUST remain non-`formal_proof` unless the referenced proof is rerun and receipt-validated by Octet.
