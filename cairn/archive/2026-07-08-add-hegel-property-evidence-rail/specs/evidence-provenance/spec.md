## ADDED Requirements

### Requirement: Valence export of Hegel property receipts
r[octet.evidence_provenance.valence_export.hegel_receipts] Octet SHOULD export verified Hegel property receipts to Valence provenance input as scoped generated-testing evidence while preserving receipt identity, parent links, source scope, and non-claim boundaries.

#### Scenario: Hegel receipts export after verification
r[octet.evidence_provenance.valence_export.hegel_receipts.export]
- GIVEN an Octet artifact directory contains Hegel property receipts that pass artifact verification and provenance validation
- WHEN the operator requests a Valence provenance export
- THEN Octet SHOULD include those receipts with property IDs, linked requirement IDs, source scope, BLAKE3 artifact identities, parent links, and generated-testing evidence tier.

#### Scenario: Hegel receipt tier is not promoted
r[octet.evidence_provenance.valence_export.hegel_receipts.no_promotion]
- GIVEN a Hegel property receipt is included in a Valence provenance export
- WHEN Octet validates the export graph
- THEN the exported tier MUST remain generated-testing or equivalent scoped validation evidence and MUST NOT become `formal_proof`.

#### Scenario: Broken Hegel receipt blocks export
r[octet.evidence_provenance.valence_export.hegel_receipts.blocked]
- GIVEN a Hegel property receipt has stale hashes, missing parent artifacts, unknown producer identity, missing non-claims, failing status presented as passing evidence, or unsupported tier promotion
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report deterministic diagnostics naming the blocking receipt.

#### Scenario: Export boundary is visible
r[octet.evidence_provenance.valence_export.hegel_receipts.boundary]
- GIVEN an operator inspects Octet's Valence provenance export documentation or Hegel receipt summary
- WHEN the Hegel integration is described
- THEN the documentation MUST state that Octet exports bounded generated-testing facts for Valence adapter consumption and does not prove whole-program correctness, verifier soundness, release eligibility, semantic equivalence, or requirement satisfaction.
