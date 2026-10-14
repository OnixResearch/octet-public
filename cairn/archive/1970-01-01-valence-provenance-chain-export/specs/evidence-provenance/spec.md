## ADDED Requirements

### Requirement: Valence provenance export
r[octet.evidence_provenance.valence_export] Octet SHOULD provide an optional export that normalizes verified Octet provenance records, proof cards, receipts, parent links, checker IDs, evidence tiers, and BLAKE3 artifact identities into adapter-ready Valence provenance input.

#### Scenario: Export is written after artifact verification
r[octet.evidence_provenance.valence_export.after_verify]
- GIVEN an Octet artifact directory passes artifact verification
- WHEN the operator requests a Valence provenance export
- THEN Octet SHOULD write a deterministic export containing the verified provenance records, proof-card links, receipt identities, parent artifacts, and diagnostics.

#### Scenario: Export is not a release claim
r[octet.evidence_provenance.valence_export.not_release_claim]
- GIVEN Octet writes a Valence provenance export
- WHEN the export is inspected
- THEN it MUST state or encode that the export is adapter input and does not by itself prove release eligibility, whole-program correctness, or Mantle bundle inclusion.

### Requirement: Valence export fail-closed behavior
r[octet.evidence_provenance.valence_export.fail_closed] Octet MUST fail closed instead of writing a Valence provenance export when Octet-owned artifact verification or linkage validation fails.

#### Scenario: Stale provenance hash blocks export
r[octet.evidence_provenance.valence_export.fail_closed.stale_hash]
- GIVEN a provenance record's BLAKE3 digest does not match the referenced artifact bytes
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report the stale artifact identity.

#### Scenario: Missing parent blocks export
r[octet.evidence_provenance.valence_export.fail_closed.missing_parent]
- GIVEN a provenance record or proof card references a parent artifact that is absent from the artifact directory
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report the missing parent artifact.

#### Scenario: Unknown checker blocks export
r[octet.evidence_provenance.valence_export.fail_closed.unknown_checker]
- GIVEN a provenance record names a checker ID that is not registered in Octet's deterministic producer registry
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report the unknown checker ID.

### Requirement: Valence provenance export CLI
r[octet.evidence_provenance.valence_export.cli] Octet SHOULD provide an operator-facing command or artifact-verifier option that writes a Valence provenance export only after the referenced artifact directory verifies successfully.

#### Scenario: CLI writes export after verification
r[octet.evidence_provenance.valence_export.cli.write]
- GIVEN an Octet artifact directory passes artifact verification
- WHEN the operator runs the Valence provenance export command or verifier flag with an output path
- THEN Octet SHOULD write the deterministic export to that path.

#### Scenario: CLI reports linkage diagnostics
r[octet.evidence_provenance.valence_export.cli.diagnostics]
- GIVEN Octet cannot export because of stale hashes, missing parents, manual-review-only evidence, or proof-card linkage failures
- WHEN the operator requests the export
- THEN Octet SHOULD report deterministic diagnostics naming the blocking evidence.

### Requirement: Evidence tier preservation in Valence export
r[octet.evidence_provenance.valence_export.tier_preservation] Octet MUST preserve evidence tiers in Valence provenance exports and MUST NOT promote validation, review, or imported evidence to `formal_proof` without an Octet-owned verifier receipt.

#### Scenario: Imported Trellis remains manual review
r[octet.evidence_provenance.valence_export.tier_preservation.trellis_import]
- GIVEN an imported Trellis proof bundle has BLAKE3 source/proof identities but has not been rerun with an Octet-owned verifier receipt
- WHEN Octet exports Valence provenance input
- THEN the exported record MUST remain external or `manual_review_required` rather than `formal_proof`.

#### Scenario: Validated proof receipt may export formal proof
r[octet.evidence_provenance.valence_export.tier_preservation.formal_proof]
- GIVEN a proof rail has an Octet-owned validated verifier receipt and matching proof package/artifact identities
- WHEN Octet exports Valence provenance input
- THEN the exported record MAY carry `formal_proof` for the scoped property and checked source/proof binding.

### Requirement: Valence export boundary documentation
r[octet.evidence_provenance.valence_export.boundary] Octet documentation MUST describe the Valence provenance export as an optional adapter input consumed by Valence stack verification, with Mantle receiving only the final sidecar as opaque external evidence.

#### Scenario: Documentation preserves dependency direction
r[octet.evidence_provenance.valence_export.boundary.dependency_direction]
- GIVEN an operator reads Octet's provenance export documentation
- WHEN the stack integration flow is described
- THEN the documentation MUST show Octet exports verified artifact facts, Valence verifies stack semantics, and Mantle bundles the final sidecar opaquely without parsing Octet or Valence semantics.
