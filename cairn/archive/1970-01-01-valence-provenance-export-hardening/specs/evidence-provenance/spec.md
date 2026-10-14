## ADDED Requirements

### Requirement: Replayable Valence export receipts
r[octet.evidence_provenance.valence_export.replayable_receipts] Octet Valence provenance exports MUST be backed by replayable receipt envelopes that bind producer identity, tool/checker/policy version, normalized invocation, input artifact IDs, output artifact IDs, parent artifacts, status, failure reason when applicable, and explicit non-claims.

#### Scenario: Passing receipt envelope is exported
r[octet.evidence_provenance.valence_export.replayable_receipts.valid]
- GIVEN an Octet artifact directory whose lint, standards, and proof records pass artifact verification
- WHEN Octet writes a Valence provenance export
- THEN each exported record MUST include the replayable receipt envelope fields and canonical BLAKE3 artifact identities.

#### Scenario: Failed receipt preserves failure reason
r[octet.evidence_provenance.valence_export.replayable_receipts.failed]
- GIVEN a scoped Octet checker receipt with a non-passing status and failure reason
- WHEN the export includes that receipt as evidence
- THEN the export MUST preserve the status and failure reason without treating it as passing evidence.

#### Scenario: Verification failure blocks export
r[octet.evidence_provenance.valence_export.replayable_receipts.blocked]
- GIVEN artifact verification, parent linkage, producer registry, or receipt envelope validation fails
- WHEN the operator requests a Valence provenance export
- THEN Octet MUST refuse to write the export and report deterministic diagnostics.

### Requirement: Typed Valence export edges
r[octet.evidence_provenance.valence_export.typed_edges] Octet Valence provenance exports MUST use typed edge kinds that describe why artifacts are linked, including source-shape checks, lint receipts, policy receipts, formal proof receipts, imported Trellis references, parent artifacts, and Valence adapter inputs.

#### Scenario: Known edge kinds pass
r[octet.evidence_provenance.valence_export.typed_edges.known]
- GIVEN export records linked by supported edge kinds
- WHEN Octet validates the export graph
- THEN validation MUST preserve the edge kind and connect the referenced artifact identities.

#### Scenario: Unsupported edge kind fails closed
r[octet.evidence_provenance.valence_export.typed_edges.unknown]
- GIVEN an export record names an edge kind outside the policy-owned vocabulary
- WHEN Octet validates the export graph
- THEN validation MUST fail with a deterministic unsupported-edge diagnostic.

#### Scenario: Edge kind does not promote tier
r[octet.evidence_provenance.valence_export.typed_edges.no_promotion]
- GIVEN a validation or review edge points at a proof-related artifact
- WHEN Octet validates the export graph
- THEN the edge kind MUST NOT promote the evidence tier beyond the producer's recorded tier.

### Requirement: Hardened evidence tier preservation
r[octet.evidence_provenance.valence_export.tier_preservation_hardened] Octet MUST preserve evidence tiers in Valence exports and MUST treat imported Trellis artifacts as manual-review or reference inputs unless Octet reruns and validates an Octet-owned proof receipt for the scoped property.

#### Scenario: Imported Trellis reference remains manual review
r[octet.evidence_provenance.valence_export.tier_preservation_hardened.trellis_manual]
- GIVEN an imported Trellis artifact identity without an Octet-owned proof receipt
- WHEN Octet exports Valence provenance input
- THEN the exported tier MUST remain manual-review or reference input rather than formal proof.

#### Scenario: Octet-owned proof receipt may be formal proof
r[octet.evidence_provenance.valence_export.tier_preservation_hardened.octet_receipt]
- GIVEN Octet reruns the proof rail and validates a passing Octet-owned proof receipt for the scoped property
- WHEN Octet exports Valence provenance input
- THEN the exported tier MAY be formal proof for that scoped property and exact content.

#### Scenario: Tier promotion fails closed
r[octet.evidence_provenance.valence_export.tier_preservation_hardened.promotion_failure]
- GIVEN an export attempts to mark manual-review or validation evidence as formal proof without an Octet-owned proof receipt
- WHEN Octet validates the export
- THEN validation MUST fail with a deterministic tier-promotion diagnostic.

### Requirement: Export fixture coverage
r[octet.evidence_provenance.valence_export.negative_fixtures] Octet MUST include positive and negative fixtures for Valence provenance export validation.

#### Scenario: Positive fixtures cover valid export families
r[octet.evidence_provenance.valence_export.negative_fixtures.positive]
- GIVEN valid lint, standards, artifact verification, and formal-proof receipt records
- WHEN the export fixture suite runs
- THEN the fixtures MUST produce deterministic adapter-ready Valence provenance input.

#### Scenario: Negative fixtures cover broken export chains
r[octet.evidence_provenance.valence_export.negative_fixtures.negative]
- GIVEN fixtures with stale digests, missing parents, unknown producers, unsupported edge kinds, missing non-claims, or manual-review tier promotion
- WHEN the export fixture suite runs
- THEN each fixture MUST fail closed with deterministic diagnostics.

### Requirement: Hardened export boundary documentation
r[octet.evidence_provenance.valence_export.boundary_hardened] Octet documentation MUST describe Valence provenance exports as verified Octet artifact facts for Valence adapter consumption, not release eligibility, Mantle bundle inclusion, whole-program correctness, or verifier soundness proof.

#### Scenario: Boundary is visible in operator docs
r[octet.evidence_provenance.valence_export.boundary_hardened.visible]
- GIVEN an operator reads Octet provenance export documentation
- WHEN the supported claim is stated
- THEN the documentation MUST say the export feeds Valence stack verification and does not itself prove release eligibility, Mantle inclusion, whole-program correctness, or verifier soundness.
