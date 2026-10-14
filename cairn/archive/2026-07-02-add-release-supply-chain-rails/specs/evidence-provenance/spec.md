## MODIFIED Requirements

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
