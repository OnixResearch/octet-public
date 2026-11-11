## ADDED Requirements

### Requirement: Structured Rollout Migration Sessions [r[structured-rollout-migration-sessions]]

The system MUST support structured rollout migration sessions that record lint rollout state as deterministic manifests rather than requiring consumers to triage an unstructured diagnostic stream. A session MUST capture profile, command inputs, config hash, tool revision, finding identities, accepted findings, autofix actions, deferred findings, and manual-review checkpoints.

#### Scenario: Migration start writes deterministic manifest [r[structured-rollout-migration-sessions.scenario.migration-start-writes-deterministic-manifest]]

- GIVEN a consumer workspace requests a starter-profile migration
- WHEN `cargo slotcar migrate start --profile starter` runs
- THEN a migration manifest is written with profile, tool revision, config hash, and initial finding identities

#### Scenario: Migration status reports remaining work [r[structured-rollout-migration-sessions.scenario.migration-status-reports-remaining-work]]

- GIVEN a migration manifest contains accepted, fixed, deferred, and unresolved findings
- WHEN `cargo slotcar migrate status --format json` runs
- THEN the output reports deterministic counts and unresolved work without reclassifying accepted findings

#### Scenario: Safe apply does not modify manual-review findings [r[structured-rollout-migration-sessions.scenario.safe-apply-does-not-modify-manual-review-findings]]

- GIVEN a migration has both machine-applicable findings and findings marked manual-review
- WHEN `cargo slotcar migrate apply-safe` runs
- THEN only machine-applicable changes are applied and manual-review findings remain pending

