## ADDED Requirements

### Requirement: Cargo-vet dependency trust receipts

r[octet.dependency_trust.vet_receipts] Octet SHOULD support optional dependency-trust receipts derived from `cargo-vet` execution or saved `cargo-vet` output.

#### Scenario: Fully covered audit passes

r[octet.dependency_trust.vet_receipts.covered]

- GIVEN `cargo-vet` reports that all configured third-party dependencies satisfy the project's audit criteria or trusted imports
- WHEN Octet records dependency-trust evidence
- THEN Octet SHOULD emit a receipt with successful status, audited package coverage, imports used, command argv, and config root.

#### Scenario: Audit gap requires review

r[octet.dependency_trust.vet_receipts.gap]

- GIVEN `cargo-vet` reports unaudited dependencies, unresolved criteria, or missing imports
- WHEN Octet records dependency-trust evidence
- THEN Octet MUST preserve the gap in the receipt.
- AND the evidence tier SHOULD be `manual_review_required` unless the configured policy explicitly treats the gap as a hard failure.

### Requirement: Dependency trust exceptions

r[octet.dependency_trust.exceptions] Dependency trust evidence MUST make exemptions and unaudited allowances explicit instead of hiding them inside a clean summary.

#### Scenario: Exemption appears in receipt

r[octet.dependency_trust.exceptions.visible]

- GIVEN dependency trust configuration includes an exemption or deferred audit allowance
- WHEN Octet emits a dependency-trust receipt
- THEN the receipt MUST include the package, version or criteria scope, rationale field when available, and review action.

#### Scenario: Silent trust import is rejected

r[octet.dependency_trust.exceptions.silent_import]

- GIVEN a dependency-trust result relies on an imported audit source
- WHEN Octet records the result
- THEN the receipt MUST identify the import source.
- AND validation MUST fail if the import source identity is absent from the receipt.

### Requirement: Dependency trust non-claims

r[octet.dependency_trust.non_claims] Dependency trust receipts MUST state that audit coverage and trust imports do not prove dependencies are benign or vulnerability-free.

#### Scenario: Receipt keeps trust boundary

r[octet.dependency_trust.non_claims.boundary]

- GIVEN a dependency-trust receipt reports successful coverage
- WHEN a reviewer inspects the receipt
- THEN the receipt MUST preserve non-claims for malicious maintainer behavior, undiscovered vulnerabilities, and project-specific business risk.

#### Scenario: Cargo-deny remains separate

r[octet.dependency_trust.non_claims.separate_from_deny]

- GIVEN both dependency-audit and dependency-trust rails are configured
- WHEN Octet renders the evidence summary
- THEN advisory/license policy results and human audit coverage MUST appear as separate evidence entries with separate checker IDs.
