# Evidence Provenance Specification Delta

## ADDED Requirements

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
