# Lint Catalog Freshness Specification

## Purpose

Defines the `lint-catalog-freshness` capability.

## Requirements

### Requirement: Catalog docs match live lint inventory

r[octet.lint_catalog_freshness.docs_match_inventory] Octet consumer-facing lint catalog documentation MUST match the live registered lint inventory for total lint count, domain counts, default levels, and listed lint identities.

#### Scenario: README count matches inventory

r[octet.lint_catalog_freshness.docs_match_inventory.readme]

- GIVEN the README describes the number of shipped Octet lints
- WHEN catalog freshness validation compares that prose with the live lint inventory
- THEN the README total MUST equal the number of registered lint inventory entries.

#### Scenario: Lint reference tables match inventory

r[octet.lint_catalog_freshness.docs_match_inventory.reference]

- GIVEN `docs/lint-reference.md` lists domain counts and lint rows
- WHEN catalog freshness validation compares the tables with live inventory and diagnostic metadata
- THEN every registered lint MUST appear exactly once with the expected domain group and default level.

### Requirement: Catalog drift fails closed

r[octet.lint_catalog_freshness.drift_detection] Octet MUST provide deterministic catalog freshness validation that fails when a consumer-facing lint catalog surface is stale.

#### Scenario: Stale count is rejected

r[octet.lint_catalog_freshness.drift_detection.count]

- GIVEN a catalog document states a lint total that differs from the live inventory
- WHEN catalog freshness validation runs
- THEN validation MUST fail with a diagnostic naming the stale total and the expected total.

#### Scenario: Missing lint row is rejected

r[octet.lint_catalog_freshness.drift_detection.missing]

- GIVEN a registered lint is absent from the lint reference catalog table
- WHEN catalog freshness validation runs
- THEN validation MUST fail with a diagnostic naming the missing lint.

#### Scenario: Mismatched default level is rejected

r[octet.lint_catalog_freshness.drift_detection.default_level]

- GIVEN a lint reference row lists a default level that differs from the registered lint default
- WHEN catalog freshness validation runs
- THEN validation MUST fail with a diagnostic naming the lint and mismatched default level.

### Requirement: Namespace-dependent prose consumes namespace contract

r[octet.lint_catalog_freshness.namespace_dependency] Catalog freshness validation SHOULD compare namespace-dependent prose against the accepted namespace compatibility contract rather than hard-coding a separate namespace decision.

#### Scenario: Namespace prose follows accepted decision

r[octet.lint_catalog_freshness.namespace_dependency.accepted]

- GIVEN the namespace compatibility contract defines the current canonical namespace and supported aliases
- WHEN catalog freshness validation checks README, lint reference, and template prose
- THEN those docs SHOULD describe the same canonical namespace and aliases.
