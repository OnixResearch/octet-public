# Release Supply Chain Specification

## Purpose

Defines the `release-supply-chain` capability.

## Requirements

### Requirement: Auditable binary release rail

r[octet.release_supply_chain.auditable_binary] Octet SHOULD provide an optional release rail that verifies Rust release binaries carry auditable dependency metadata when the consumer selects that mode.

#### Scenario: Auditable binary passes

r[octet.release_supply_chain.auditable_binary.passes]

- GIVEN a release binary was built with auditable dependency metadata and the selected binary-audit command can recover the dependency list
- WHEN Octet records release supply-chain evidence
- THEN the receipt SHOULD name the binary path, recovered package count, checker ID, command argv, and result status.

#### Scenario: Missing dependency metadata fails selected rail

r[octet.release_supply_chain.auditable_binary.missing]

- GIVEN the auditable-binary rail is selected for a release artifact
- WHEN the selected checker cannot recover required Rust dependency metadata
- THEN Octet MUST fail the rail with a diagnostic naming the artifact and missing metadata class.

### Requirement: SBOM and vulnerability scan receipts

r[octet.release_supply_chain.sbom_scan] Octet SHOULD support optional SBOM generation and vulnerability scan receipts for release artifacts, filesystems, or container images.

#### Scenario: SBOM receipt links to scan receipt

r[octet.release_supply_chain.sbom_scan.parent_link]

- GIVEN Octet generates an SBOM and then scans that SBOM for vulnerabilities
- WHEN provenance is emitted
- THEN the scan receipt SHOULD list the SBOM receipt as a parent artifact.
- AND both receipts SHOULD include BLAKE3 content hashes and checked scope.

#### Scenario: Vulnerability finding is not hidden

r[octet.release_supply_chain.sbom_scan.finding]

- GIVEN the vulnerability scanner reports a finding above the configured release threshold
- WHEN Octet renders the release supply-chain summary
- THEN the summary MUST preserve the finding severity, package identity, scanner identity, and configured gate outcome.

### Requirement: Supply-chain attestation compatibility

r[octet.release_supply_chain.attestation_compat] Octet release supply-chain receipts SHOULD be shaped for later SLSA or in-toto compatibility without requiring hosted builders, signing services, or transparency logs.

#### Scenario: Local receipt has attestation-shaped fields

r[octet.release_supply_chain.attestation_compat.fields]

- GIVEN Octet emits release supply-chain evidence
- WHEN the receipt is serialized
- THEN it SHOULD include subject artifact identity, materials or parent artifacts, builder or runner identity, command argv, result status, and checked scope.

#### Scenario: Hosted attestation remains optional

r[octet.release_supply_chain.attestation_compat.local_first]

- GIVEN a controlled-deployment consumer runs release supply-chain rails locally
- WHEN receipts are emitted
- THEN Octet MUST NOT require Sigstore, hosted SLSA builders, external transparency logs, or uploading source or artifacts.
- AND the receipt MUST NOT claim SLSA compliance unless a separate accepted contract validates that claim.
