## ADDED Requirements

### Requirement: Cargo-deny dependency audit rail

r[octet.dependency_audit.cargo_deny_rail] Octet SHOULD provide a first-class dependency-audit rail backed by `cargo-deny` for Cargo workspaces that opt into dependency policy evidence.

#### Scenario: Clean dependency policy passes

r[octet.dependency_audit.cargo_deny_rail.clean]

- GIVEN a consumer workspace has a checked-in `deny.toml` and dependency graph that satisfies the configured advisory, license, ban, duplicate, and source rules
- WHEN the dependency-audit rail runs through Octet
- THEN the rail SHOULD exit successfully.
- AND the report SHOULD identify `cargo-deny` as the checker, the workspace scope, the config path, and the checked rule families.

#### Scenario: Denied dependency policy fails

r[octet.dependency_audit.cargo_deny_rail.denied]

- GIVEN `cargo-deny` reports a denied advisory, license, duplicate crate, banned crate, or untrusted source
- WHEN Octet evaluates the dependency-audit rail
- THEN Octet MUST fail the rail with a deterministic diagnostic that preserves the `cargo-deny` finding category.
- AND Octet MUST NOT rewrite the finding as an Octet source lint.

### Requirement: Dependency-audit config and update boundary

r[octet.dependency_audit.config_boundary] Dependency-audit execution MUST make the config path, lockfile scope, and advisory-database update behavior explicit.

#### Scenario: Locked evidence avoids implicit updates

r[octet.dependency_audit.config_boundary.locked]

- GIVEN a release or evidence command requests locked dependency-audit evidence
- WHEN Octet runs the dependency-audit rail
- THEN Octet MUST NOT implicitly update advisory databases or policy inputs.
- AND the receipt MUST state whether the advisory database was reused, updated by an explicit prior step, or unavailable.

#### Scenario: Missing config is actionable

r[octet.dependency_audit.config_boundary.missing_config]

- GIVEN a workspace does not provide a dependency-audit config and no Octet template path is supplied
- WHEN the dependency-audit rail is requested
- THEN Octet MUST fail closed with a diagnostic naming the missing config path and the documented template or setup command.

### Requirement: Dependency-audit receipts and provenance

r[octet.dependency_audit.receipt] Octet dependency-audit runs SHOULD emit deterministic summary and JSON receipt artifacts with `policy_check` provenance.

#### Scenario: Receipt names policy evidence fields

r[octet.dependency_audit.receipt.fields]

- GIVEN the dependency-audit rail completes
- WHEN Octet writes its artifact directory
- THEN the receipt SHOULD include schema version, checker ID, command argv, workspace scope, config path, lockfile identity, checked rule families, result status, finding counts by family, and bounded log identity.
- AND provenance SHOULD include source ID, checker ID, evidence tier `policy_check`, artifact locator, BLAKE3 content hash, and checked scope.

#### Scenario: Receipt does not give legal advice

r[octet.dependency_audit.receipt.no_legal_advice]

- GIVEN the dependency-audit rail reports license findings
- WHEN Octet renders human or JSON output
- THEN the output MUST state policy pass/fail and finding metadata.
- AND the output MUST NOT claim legal approval or license compatibility beyond the configured policy result.
