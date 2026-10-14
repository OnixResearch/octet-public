## Phase 1: Implementation

- [x] [serial] r[octet.proof_claim_catalog.stable_ids] Inventory existing proof/property IDs, proof-card IDs, proof-ref fields, evidence source IDs, and documentation anchors that behave like named claims.
- [x] [serial] r[octet.proof_claim_catalog.stable_ids] Add a typed catalog input and deterministic generated artifact for selected cross-surface claims.
- [x] [serial] r[octet.proof_claim_catalog.linkage] Implement pure reference validation from catalog entries to proof cards, proof refs, receipts, provenance records, and documentation anchors where applicable.
- [x] [serial] r[octet.proof_claim_catalog.aliases] Add explicit alias/deprecation handling for renamed claims.
- [x] [serial] r[octet.proof_claim_catalog.boundaries] Document catalog non-claims and evidence-tier boundaries.

## Phase 2: Verification

- [x] [serial] r[octet.proof_claim_catalog.stable_ids.unique] Verify valid catalog entries export deterministically and duplicate IDs fail closed with diagnostics naming the duplicated claim.
- [x] [serial] r[octet.proof_claim_catalog.linkage.valid] Verify a cataloged claim can be linked to matching proof-card, proof-ref, receipt, provenance, or documentation evidence without changing public wire shapes.
- [x] [serial] r[octet.proof_claim_catalog.linkage.stale] Verify missing linked artifacts, stale source anchors, invalid evidence-tier promotion, and unknown requirement or policy IDs fail closed.
- [x] [serial] r[octet.proof_claim_catalog.aliases.explicit] Verify aliases resolve only when explicitly declared and incompatible claim replacement fails validation.
- [x] [serial] r[octet.proof_claim_catalog.boundaries.no_overclaim] Verify catalog wording rejects whole-program correctness, certification, business-domain validation, semantic-equivalence, full-safety, and verified-runtime overclaims.
- [x] [serial] Run catalog freshness checks, focused proof-card/proof-ref tests, and Cairn validate plus proposal/design/tasks gates. [depends:cairn-gates]
