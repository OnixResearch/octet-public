# Coverage Baseline Evidence Specification

## Purpose

Defines the `coverage-baseline-evidence` capability.

## Requirements

### Requirement: Coverage runs use typed bounded profiles

r[octet.coverage_baseline.profile]

Octet MUST run cargo-llvm-cov only through a validated Nickel profile that binds provider, Rust and LLVM toolchains, Nix closure, scope, features, tests, metrics, path policy, fixed argv, thresholds, limits, and non-claims.

#### Scenario: Reviewed profile constructs a run

- GIVEN a complete profile selects supported scope, provider, toolchains, metrics, thresholds, limits, and non-claims
- WHEN coverage run preflight executes
- THEN Octet MUST construct the declared argv deterministically
- AND the receipt MUST bind the profile, lockfile, toolchain, provider, source, and argv identities

#### Scenario: Arbitrary run authority fails

- GIVEN an operator supplies free-form command text, undeclared Cargo arguments, unsafe paths, ambient environment inputs, or a network-dependent provider
- WHEN preflight executes
- THEN Octet MUST reject the request before process execution

### Requirement: Coverage evidence uses native count artifacts

r[octet.coverage_baseline.native_artifacts]

Octet MUST normalize covered and total counts from supported pinned cargo-llvm-cov machine output and MUST retain every raw report identity beside the normalization version.

#### Scenario: Supported native report normalizes

- GIVEN a pinned run writes a complete supported report to the declared capability-rooted path
- WHEN the pure normalization core runs twice
- THEN both normalized count projections and diagnostic orderings MUST match
- AND each enabled metric MUST retain covered and total counts

#### Scenario: Report contains only percentages or human logs

- GIVEN a run supplies caller-calculated percentages, mixed terminal output, or an unsupported native report version without required counts
- WHEN normalization runs
- THEN Octet MUST fail with a missing-count or unsupported-provider diagnostic
- AND it MUST NOT infer passing coverage from the supplied percentages or logs

### Requirement: Coverage comparison is internal and scope-safe

r[octet.coverage_baseline.comparison]

Octet MUST calculate threshold failures and regressions from verified current and baseline counts only when their collection and source scopes are comparable.

#### Scenario: Comparable reports produce a decision

- GIVEN current and baseline reports match profile, toolchain, target, feature, test-selection, metric, path-policy, and source-scope identities
- WHEN comparison runs
- THEN Octet MUST calculate count deltas, threshold outcomes, and regression outcomes deterministically
- AND display percentages MUST follow the declared precision and rounding policy

#### Scenario: Reports are not comparable

- GIVEN one required collection or source identity differs or is missing
- WHEN comparison runs
- THEN Octet MUST fail with a deterministic comparability diagnostic
- AND it MUST NOT report an improvement, regression, or passing baseline decision

#### Scenario: Denominator changes

- GIVEN covered counts and total counts change between baseline and current reports
- WHEN comparison runs
- THEN the receipt MUST preserve both count pairs and the selected denominator-change policy
- AND a rounded percentage alone MUST NOT determine the result

### Requirement: Coverage receipts bind current and baseline evidence

r[octet.coverage_baseline.receipt]

A hardened coverage receipt MUST bind the current report, accepted baseline, profile, normalized counts, comparison, thresholds, regressions, logs, paths, parent links, and BLAKE3 identities.

#### Scenario: Complete receipt verifies

- GIVEN current and baseline native reports are valid, comparable, and within limits
- WHEN artifact validation runs
- THEN validation MUST remeasure every report and projection identity
- AND it MUST reproduce the comparison decision from bound counts and policy

#### Scenario: Baseline or projection is stale

- GIVEN the baseline, current report, normalized projection, profile, path mapping, or comparison changes after collection
- WHEN artifact validation runs
- THEN validation MUST fail with a deterministic identity diagnostic

### Requirement: Coverage remains bounded execution evidence

r[octet.coverage_baseline.boundary]

Coverage receipts MUST remain optional `transcript_assertion` evidence and MUST NOT claim behavioral correctness, path correctness, test completeness, formal proof, or release eligibility.

#### Scenario: Compatible import remains bounded

- GIVEN an existing normalized `cargo-llvm-cov-summary/v1` input passes its accepted contract
- WHEN Octet imports it without verified native current and baseline parents
- THEN the receipt MUST remain legacy imported transcript evidence
- AND it MUST NOT claim Octet-calculated baseline comparison

#### Scenario: Starter checks run

- GIVEN no coverage profile is selected
- WHEN starter `cargo octet check` runs
- THEN cargo-llvm-cov and its artifacts MUST NOT be required

### Requirement: Coverage hardening has positive and negative verification

r[octet.coverage_baseline.verification]

The implementation MUST include deterministic positive and negative fixtures for profiles, native reports, counts, paths, comparability, thresholds, regressions, identities, limits, compatibility, and non-claims.

#### Scenario: Focused coverage matrix completes

- GIVEN stable, improved, threshold, regression, denominator, imported, incomparable, malformed, stale, unsafe, over-limit, tool-failure, and overclaim fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
