# Self Check Specification

## Purpose

Defines the `self-check` capability.

## Requirements

### Requirement: Self-check warning budget is refreshed after reviewed source changes

r[octet.self_check.warning_budget_refresh] The canonical self-check baseline
and its accepted identity/warning totals MUST be refreshed together when the
reviewed workspace finding set changes. The refresh MUST preserve the baseline
schema, finding schema, stable identity shape, and explicit advisory non-claim.

#### Scenario: Refreshed baseline has no stale or new findings

- GIVEN the refreshed baseline and the current workspace
- WHEN the canonical self-check runs
- THEN it reports zero new and zero stale findings.

#### Scenario: Counts match the baseline

- GIVEN the refreshed baseline file
- WHEN the self-check derivation validates it
- THEN the accepted identity and warning totals match the file and structured
  output.

#### Scenario: Warning budget remains advisory

- GIVEN the refreshed baseline
- WHEN the claim boundary is reviewed
- THEN the baseline is not presented as correctness or release evidence.
