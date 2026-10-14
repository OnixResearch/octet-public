# Evidence Provenance Specification

## Purpose

Defines additions to the `evidence-provenance` capability.

## Requirements

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

r[octet.evidence_provenance.parent_chain] Provenance records for derived evidence SHOULD identify parent artifacts when the derived artifact summarizes, bundles, or gates earlier Octet artifacts.

#### Scenario: Derived artifact links parents

r[octet.evidence_provenance.parent_chain.links]

- GIVEN a policy bundle, release smoke receipt, or evidence summary is derived from earlier Octet artifacts
- WHEN its provenance record is emitted
- THEN the record SHOULD list parent artifact locators or identities for the artifacts it summarizes.

#### Scenario: Invalid parent chain fails

r[octet.evidence_provenance.parent_chain.invalid]

- GIVEN a provenance record references a missing parent artifact or a set of records forms a parent cycle
- WHEN parent chain validation runs
- THEN validation MUST fail with a diagnostic naming the missing parent or cycle.
