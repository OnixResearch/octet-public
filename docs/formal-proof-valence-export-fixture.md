# Formal-proof Valence export fixture

The formal-proof Valence export fixture is a frozen, hermetic adapter fixture for scoped Verus proof evidence. It ties together a passed Octet-owned proof receipt, proof card, formal-proof provenance record, artifact verification evidence, and Valence adapter export row.

Validation requires:

- a passed scoped proof receipt;
- a proof card linked to the same receipt and verifier identity;
- current artifact verification evidence;
- matching `formal_proof` tier across the Octet receipt/provenance and Valence export;
- required non-claims.

Imported Trellis/manual-review references, lint/policy/Hegel/Kani evidence, stale proof cards, skipped verification, missing non-claims, and release/correctness overclaims fail closed. The fixture proves export shape and linkage only; Octet proof receipts remain authoritative and the Valence export is adapter input, not an independent proof rail or release decision.
