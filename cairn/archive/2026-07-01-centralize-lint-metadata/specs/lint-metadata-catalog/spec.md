## ADDED Requirements

### Requirement: Rust-owned lint metadata catalog

r[octet.lint_metadata.single_source] Octet MUST maintain one Rust-owned lint metadata catalog that covers every registered lint identity, domain group, default level, guidance text, effect family, documentation summary, and canonical namespace decision.

#### Scenario: Registered lint has catalog metadata

r[octet.lint_metadata.single_source.registered]

- GIVEN a lint appears in the runtime lint inventory
- WHEN lint metadata freshness validation runs
- THEN validation MUST find exactly one catalog entry for that lint.
- AND the catalog entry MUST name the lint's group, default level, principle, guidance, effect family, and documentation summary.

#### Scenario: Unknown catalog entry fails

r[octet.lint_metadata.single_source.unknown]

- GIVEN the metadata catalog contains an entry that is not registered as a lint
- WHEN lint metadata freshness validation runs
- THEN validation MUST fail with a diagnostic naming the stale catalog entry.

### Requirement: Runner metadata coverage

r[octet.lint_metadata.runner_coverage] `cargo-octet` machine-readable findings and SARIF output MUST derive lint metadata from the Rust-owned catalog or fail freshness validation when any registered lint lacks runner metadata.

#### Scenario: Sidecar metadata is complete

r[octet.lint_metadata.runner_coverage.complete]

- GIVEN `cargo-octet` emits a finding for any registered Octet lint
- WHEN JSONL or SARIF output is rendered
- THEN the finding MUST include the catalog group, principle, guidance, and effect family for that lint.

#### Scenario: Missing runner metadata fails

r[octet.lint_metadata.runner_coverage.missing]

- GIVEN a registered lint has no runner metadata mapping
- WHEN runner metadata validation runs
- THEN validation MUST fail before release evidence can be accepted.

### Requirement: Catalog-derived docs freshness

r[octet.lint_metadata.docs_freshness] Consumer-facing lint documentation and rollout templates SHOULD be generated from or deterministically checked against the Rust-owned lint catalog.

#### Scenario: Lint reference matches catalog

r[octet.lint_metadata.docs_freshness.reference]

- GIVEN the lint reference is checked in
- WHEN docs freshness validation runs
- THEN the documented lint count, domain counts, default levels, and lint summaries SHOULD match the Rust-owned catalog.

#### Scenario: Namespace prose matches catalog

r[octet.lint_metadata.docs_freshness.namespace]

- GIVEN README, lint reference, and templates describe source-attribute namespaces
- WHEN docs freshness validation runs
- THEN the described canonical namespace and compatibility aliases SHOULD match the catalog namespace decision.
