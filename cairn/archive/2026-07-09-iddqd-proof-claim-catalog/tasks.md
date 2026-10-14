## Tasks

- [x] [serial] Add iddqd to the relevant Octet crate and define ordered proof-claim and alias index helpers. r[octet.verus_proof_rails.iddqd_proof_claim_catalog.index]
- [x] [serial] Refactor claim and alias validation to build unique identity indexes before reference resolution. r[octet.verus_proof_rails.iddqd_proof_claim_catalog.validation]
- [x] [parallel] Add positive fixtures for canonical claims, alias claims, deprecated aliases, required artifacts, and documentation anchors. r[octet.verus_proof_rails.iddqd_proof_claim_catalog.fixtures.positive]
- [x] [parallel] Add negative fixtures for duplicate claim ids, duplicate aliases, claim/alias conflicts, self-targeting aliases, missing targets, invalid tiers, missing artifacts, missing anchors, and weakened non-claims. r[octet.verus_proof_rails.iddqd_proof_claim_catalog.fixtures.negative]
- [x] [serial] Document why aliases are not modeled as a bijection and keep proof-card/proof-ref artifact shapes stable. r[octet.verus_proof_rails.iddqd_proof_claim_catalog.docs]
- [x] [serial] Run focused proof claim tests, cargo octet artifact/proof checks as applicable, and Cairn validation/gates. r[octet.verus_proof_rails.iddqd_proof_claim_catalog.final_validation]
