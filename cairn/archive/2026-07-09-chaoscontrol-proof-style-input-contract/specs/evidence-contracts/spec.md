# Evidence Contracts Specification

## Purpose

Adds proof-style input contracts for ChaosControl consumer gates.

## Requirements

### Requirement: ChaosControl proof-style input contract
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.contract] Octet MUST support a contract for ChaosControl proof-style input rails that records selected profile, focused crates, policy registry identity, input revisions, commands, and non-claims.

#### Scenario: Current input contract passes
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.fixtures.positive]
- GIVEN a contract row names ChaosControl, an Octet profile, focused crates, registry digest, input revision, command, and required non-claims
- WHEN validation runs
- THEN the row MUST pass and preserve the consumer/profile identity.

#### Scenario: Stale input contract fails
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.fixtures.negative]
- GIVEN a contract row has stale input revision, missing profile, missing focused crate, or overbroad proof-style claim
- WHEN validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Contract validation
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.validation] Octet MUST validate proof-style input contracts without claiming ChaosControl runtime determinism or VMM correctness.

#### Scenario: Boundary is visible
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.docs]
- GIVEN a ChaosControl proof-style input contract passes
- WHEN the supported claim is rendered
- THEN it MUST state that Octet validates profile/input identity only and does not prove ChaosControl behavioral correctness.

### Requirement: Final validation
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers contract health
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.final_validation.fixtures]
- GIVEN valid and invalid input contract fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
