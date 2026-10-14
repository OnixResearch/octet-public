## ADDED Requirements

### Requirement: Clippy co-reporting rail

r[octet.external_lint.clippy_co_reporting] Octet SHOULD provide an optional Clippy co-reporting rail that can run Clippy or import Clippy JSON while preserving Clippy's diagnostic identity.

#### Scenario: Clippy findings are preserved

r[octet.external_lint.clippy_co_reporting.preserved]

- GIVEN Clippy emits a lint finding with a lint name, level, message, span, and suggestion metadata
- WHEN Octet imports or co-runs Clippy
- THEN Octet SHOULD preserve those fields in the external-lint sidecar.
- AND Octet MUST NOT relabel the finding as an Octet lint ID.

#### Scenario: Clean Clippy run joins artifact bundle

r[octet.external_lint.clippy_co_reporting.clean]

- GIVEN a consumer opts into Clippy co-reporting and Clippy exits successfully
- WHEN Octet writes an artifact directory
- THEN the artifact directory SHOULD include a deterministic Clippy summary or receipt.
- AND artifact verification SHOULD cover that receipt when present.

### Requirement: External lint severity mapping

r[octet.external_lint.severity_mapping] External lint reporting MUST preserve source checker severity while making Octet gate behavior explicit.

#### Scenario: Source and gate levels are distinct

r[octet.external_lint.severity_mapping.distinct]

- GIVEN a Clippy finding has a source lint level and Octet applies a separate gate policy
- WHEN Octet renders the finding
- THEN the report MUST distinguish the Clippy source level from the Octet gate outcome.

#### Scenario: Import failure fails deterministically

r[octet.external_lint.severity_mapping.import_failure]

- GIVEN a Clippy JSON report is malformed or does not match the expected message format
- WHEN Octet imports the report
- THEN Octet MUST fail with a diagnostic naming the malformed input and field.

### Requirement: External lint provenance

r[octet.external_lint.provenance] External Clippy evidence SHOULD use `static_lint` provenance with a registered Clippy checker identity.

#### Scenario: Provenance identifies Clippy

r[octet.external_lint.provenance.checker]

- GIVEN Octet emits a Clippy co-reporting artifact
- WHEN provenance is written
- THEN the provenance record SHOULD name `clippy` or the registered Clippy checker ID, evidence tier `static_lint`, artifact locator, BLAKE3 content hash, and checked scope.

#### Scenario: Hosted scanners stay optional

r[octet.external_lint.provenance.no_hosted_requirement]

- GIVEN Clippy co-reporting is enabled
- WHEN the rail runs in a controlled-deployment environment
- THEN it MUST NOT require SonarQube, CodeQL, Semgrep, GitHub code scanning, or any hosted service.
