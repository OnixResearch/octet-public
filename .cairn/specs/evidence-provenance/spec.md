# Evidence Provenance Specification

## Purpose

Defines the `evidence-provenance` capability.

## Requirements

### Requirement: Provenance record

r[octet.evidence_provenance.record] Octet evidence summaries that claim requirement or policy coverage SHOULD include a provenance record with source ID, checker ID, evidence tier, artifact locator, BLAKE3 content hash, and checked scope.

#### Scenario: Provenance links source to artifact

r[octet.evidence_provenance.record.links]

- GIVEN an evidence artifact claims support for a requirement, policy, lint, or transcript assertion
- WHEN the artifact summary is emitted
- THEN the summary SHOULD include a source ID, checker ID, evidence tier, artifact locator, BLAKE3 content hash, and checked scope.
- AND a reviewer MUST be able to identify the originating requirement or policy from the record.

### Requirement: Provenance validation fails closed

r[octet.evidence_provenance.fail_closed] Provenance validation MUST fail closed when required source identity, checker identity, or content identity is missing or inconsistent.

#### Scenario: Missing source ID fails

r[octet.evidence_provenance.fail_closed.missing_source]

- GIVEN a provenance record omits the source ID
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the missing source ID field.

#### Scenario: Unknown checker ID fails

r[octet.evidence_provenance.fail_closed.unknown_checker]

- GIVEN a provenance record names a checker ID that is not known to the registry
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the unknown checker ID.

#### Scenario: Hash mismatch fails

r[octet.evidence_provenance.fail_closed.hash_mismatch]

- GIVEN a provenance record contains a BLAKE3 hash that does not match the referenced deterministic artifact
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the artifact locator.

### Requirement: Provenance registry ownership

r[octet.evidence_provenance.registry] The provenance record family MUST be registered in `standards/policies/registry.ncl` under the correct ownership classification.

#### Scenario: Registry covers provenance records

r[octet.evidence_provenance.registry.covered]

- GIVEN Octet emits provenance records after execution
- WHEN artifact manifest freshness validation runs
- THEN the provenance family MUST be covered as Rust-derived unless an explicit non-candidate rationale applies.

### Requirement: Cross-gate provenance coverage

r[octet.evidence_provenance.cross_gate_coverage] Octet SHOULD use the same provenance record model for deterministic evidence emitted by runner artifact bundles, standards policy checks, fingerprint review manifests, transcript assertions, and generated policy artifacts.

#### Scenario: Release chain names every producer

r[octet.evidence_provenance.cross_gate_coverage.release_chain]

- GIVEN a release operator gathers Octet runner, standards, policy, fingerprint, and transcript artifacts
- WHEN provenance records are emitted or verified for those artifacts
- THEN each deterministic artifact SHOULD have a record with a source ID, checker ID, evidence tier, artifact locator, BLAKE3 hash, and checked scope.
- AND the evidence tier MUST match the producer's evidence type rather than promoting validation or review artifacts to formal proof.

### Requirement: Provenance producer registry

r[octet.evidence_provenance.producer_registry] Provenance validation MUST use a deterministic registry of known checker IDs and source ID families for Octet evidence producers.

#### Scenario: Known producers pass

r[octet.evidence_provenance.producer_registry.known]

- GIVEN provenance records name registered Octet checker IDs and accepted source ID families
- WHEN producer registry validation runs
- THEN validation MUST accept those producer identities.

#### Scenario: Unknown producer fails

r[octet.evidence_provenance.producer_registry.unknown]

- GIVEN a provenance record names an unknown checker ID or unsupported source ID family
- WHEN producer registry validation runs
- THEN validation MUST fail with a diagnostic naming the unknown producer field.

### Requirement: Parent artifact chains

r[octet.evidence_provenance.parent_chain] Provenance records for derived evidence SHOULD identify parent artifacts when the derived artifact summarizes, bundles, scans, attests to, or gates earlier Octet artifacts.

#### Scenario: Derived artifact links parents

r[octet.evidence_provenance.parent_chain.links]

- GIVEN a policy bundle, release smoke receipt, SBOM scan receipt, release supply-chain receipt, or evidence summary is derived from earlier Octet artifacts
- WHEN its provenance record is emitted
- THEN the record SHOULD list parent artifact locators or identities for the artifacts it summarizes.

#### Scenario: Invalid parent chain fails

r[octet.evidence_provenance.parent_chain.invalid]

- GIVEN a provenance record references a missing parent artifact or a set of records forms a parent cycle
- WHEN parent chain validation runs
- THEN validation MUST fail with a diagnostic naming the missing parent or cycle.

### Requirement: Octet evidence-chain handoffs are typed

r[octet.evidence_provenance.evidence_chain_handoff.contract] Octet MUST provide a generic evidence-chain producer handoff envelope with typed evidence kind, producer identity, run identity, canonical roots, parent links, policy/profile hashes, projections, and non-claim boundaries.

#### Scenario: Function-address profile uses generic handoff
r[octet.evidence_provenance.evidence_chain_handoff.function_address]
- GIVEN Octet exports function-address evidence
- WHEN the export is marked for stack-chain consumption
- THEN Octet MUST wrap the function-address profile payload in the generic producer handoff envelope.

### Requirement: Handoff validation fails closed

r[octet.evidence_provenance.evidence_chain_handoff.core] Octet MUST validate evidence-chain handoff fields before treating an output as ready for Kamacite canonicalization.

#### Scenario: Invalid handoff is rejected
r[octet.evidence_provenance.evidence_chain_handoff.negative]
- GIVEN a producer handoff omits evidence kind, uses an untyped parent hash, contains malformed BLAKE3, lacks policy identity, duplicates a canonical root, treats JSON as authority, or weakens non-claims
- WHEN handoff validation runs
- THEN Octet MUST emit deterministic diagnostics and MUST NOT promote the handoff as stack-chain ready.

### Requirement: Compatibility projections are non-authoritative

r[octet.evidence_provenance.evidence_chain_handoff.projection] Octet MAY emit JSON projections for evidence-chain profiles, but MUST document that Kamacite canonical receipts own portable stack identity.

#### Scenario: Valid handoff feeds Kamacite
r[octet.evidence_provenance.evidence_chain_handoff.positive]
- GIVEN a valid Octet evidence-chain producer handoff
- WHEN Kamacite canonicalizes the handoff
- THEN the resulting receipt MUST preserve Octet's typed parent links and non-claim boundary.

### Requirement: Stack smoke validates the first profile

r[octet.evidence_provenance.evidence_chain_handoff.validation] Octet SHOULD validate the generic handoff through the function-address stack smoke before additional profiles depend on it.

#### Scenario: Smoke proves profile-neutral envelope
- GIVEN function-address is the first evidence-chain profile
- WHEN the stack smoke runs through Octet, Kamacite, Valence, Mantle, and Cairn
- THEN the smoke MUST prove the generic envelope can carry one profile without profile-specific wrapper code.

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

### Requirement: Preserves receipt export contract

r[octet.evidence_provenance.preserves_receipt_export.contract] Octet MUST support an optional Preserves receipt envelope export contract that links canonical Preserves envelopes to verified Octet source receipts, rails, roles, consumer profiles, and non-claims.

#### Scenario: Verified Octet receipt envelope passes

r[octet.evidence_provenance.preserves_receipt_export.fixtures.positive]

- GIVEN an Octet artifact directory has passed artifact verification and an export row records source receipt digest, rail, schema label, canonical Preserves digest, evidence role, consumer profile, and required non-claims
- WHEN export validation runs
- THEN validation MUST pass and preserve source receipt linkage.

#### Scenario: Invalid Octet receipt envelope fails

r[octet.evidence_provenance.preserves_receipt_export.fixtures.negative]

- GIVEN an export row has missing source receipt, stale digest, skipped verification, role promotion, unsupported schema, or missing non-claims
- WHEN export validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Export validation boundary

r[octet.evidence_provenance.preserves_receipt_export.validation] Octet MUST treat Preserves export as an adapter over already verified Octet artifacts, not as a new proof rail or source-of-truth receipt.

#### Scenario: Adapter boundary is visible

r[octet.evidence_provenance.preserves_receipt_export.docs]

- GIVEN a Preserves export validates
- WHEN the supported claim is rendered
- THEN it MUST state that Octet source receipts remain authoritative and the Preserves envelope only carries identity/linkage for consumers.

### Requirement: Final validation

r[octet.evidence_provenance.preserves_receipt_export.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers export health

r[octet.evidence_provenance.preserves_receipt_export.final_validation.fixtures]

- GIVEN valid and invalid Preserves export fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.

### Requirement: Formal proof Valence export fixture contract
r[octet.evidence_provenance.formal_proof_valence_export_fixture.contract] Octet MUST define a frozen fixture contract for Valence provenance exports produced from scoped validated formal-proof artifact bundles.

#### Scenario: Fixture contract names proof source and export target
r[octet.evidence_provenance.formal_proof_valence_export_fixture.contract.fields]
- GIVEN a formal-proof Valence export fixture is declared
- WHEN Octet validates the fixture contract
- THEN the fixture MUST name the proof receipt, proof card, provenance record, artifact verification evidence, Valence export target, evidence tier, typed edges, and required non-claim boundary.

### Requirement: Positive formal-proof export fixture
r[octet.evidence_provenance.formal_proof_valence_export_fixture.positive] Octet MUST provide a positive fixture where a passed scoped Verus proof receipt, proof card, provenance record, artifact verification report, and Valence export agree on proof identity.

#### Scenario: Verified proof export passes
r[octet.evidence_provenance.formal_proof_valence_export_fixture.positive.verified]
- GIVEN a scoped Verus proof receipt passed, its proof card links the same receipt and verifier identities, artifact verification succeeded, and the Valence export preserves the same formal-proof provenance
- WHEN Octet validates the export fixture
- THEN validation MUST pass and preserve the `formal_proof` evidence tier only for that scoped proof record.

### Requirement: Negative formal-proof export fixtures
r[octet.evidence_provenance.formal_proof_valence_export_fixture.negative] Octet MUST reject formal-proof export fixtures with missing receipts, stale proof-card links, skipped artifact verification, tier promotion, missing non-claims, or overclaim text.

#### Scenario: Export cannot bypass artifact verification
r[octet.evidence_provenance.formal_proof_valence_export_fixture.negative.skipped_verify]
- GIVEN a Valence export row claims formal-proof evidence but lacks current artifact verification evidence
- WHEN Octet validates the export fixture
- THEN validation MUST fail with a deterministic skipped-verification diagnostic.

#### Scenario: Manual-review import cannot become formal proof
r[octet.evidence_provenance.formal_proof_valence_export_fixture.negative.manual_review]
- GIVEN an imported Trellis proof reference or manual-review record is marked `formal_proof` without an Octet-owned rerun receipt
- WHEN Octet validates the export fixture
- THEN validation MUST fail with a deterministic tier-promotion diagnostic.

### Requirement: Formal-proof export validation core
r[octet.evidence_provenance.formal_proof_valence_export_fixture.validation] Octet MUST validate formal-proof Valence export fixtures through a pure decision core fed by shell-loaded fixture data.

#### Scenario: Pure validator owns eligibility decisions
r[octet.evidence_provenance.formal_proof_valence_export_fixture.validation.core]
- GIVEN parsed fixture rows and expected identity values
- WHEN fixture validation runs
- THEN the pure validator MUST decide pass/fail diagnostics without performing filesystem, process, network, or environment access.

### Requirement: Export boundary documentation
r[octet.evidence_provenance.formal_proof_valence_export_fixture.docs] Octet documentation MUST state that formal-proof Valence exports are adapter inputs, not independent proof rails or release decisions.

#### Scenario: Adapter non-claim is visible
r[octet.evidence_provenance.formal_proof_valence_export_fixture.docs.non_claims]
- GIVEN a formal-proof Valence export fixture validates
- WHEN an operator reads the supported claim
- THEN the report MUST state that Octet proof receipts remain authoritative and the Valence export only carries identity and linkage for stack consumers.

### Requirement: Final validation evidence
r[octet.evidence_provenance.formal_proof_valence_export_fixture.final_validation] The change MUST include positive and negative fixtures, artifact verification checks, policy validation, and Cairn validation evidence before archive.

#### Scenario: Fixture matrix covers export health
r[octet.evidence_provenance.formal_proof_valence_export_fixture.final_validation.fixtures]
- GIVEN valid and invalid formal-proof Valence export fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, and saved evidence MUST bind fixture, artifact, and policy identities.
