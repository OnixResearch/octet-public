## ADDED Requirements

### Requirement: Advisory lint promotion evidence gate

r[octet.advisory_lint_promotion.evidence_gate] Octet MUST require saved evidence before an allow-by-default lint is promoted to a starter profile, boundary profile, or global `warn` default.

#### Scenario: Promotion evidence is complete

r[octet.advisory_lint_promotion.evidence_gate.complete]

- GIVEN an allow-by-default lint is proposed for promotion
- WHEN maintainers review the promotion change
- THEN the change MUST include dogfood or downstream finding counts, false-positive triage, representative fixes, suppression compatibility proof, and a final recommendation.

#### Scenario: Profile promotion is distinct from default promotion

r[octet.advisory_lint_promotion.evidence_gate.profile]

- GIVEN a lint is ready for targeted rollout but not global default promotion
- WHEN the change updates rollout profiles
- THEN the evidence MUST state that only the profile changed and the lint default remains unchanged.

### Requirement: Promotion migration and rollback notes

r[octet.advisory_lint_promotion.migration] Promoted lints MUST include migration notes and rollback guidance for consumers.

#### Scenario: Default-level snapshot changes are explicit

r[octet.advisory_lint_promotion.migration.default_level]

- GIVEN a lint default level changes
- WHEN catalog freshness and default-level snapshot tests run
- THEN the snapshot diff MUST name the changed lint and the new level.
- AND README, lint reference, and templates MUST describe the consumer-visible migration.

#### Scenario: Suppression compatibility remains tested

r[octet.advisory_lint_promotion.migration.suppression]

- GIVEN a promoted lint can be suppressed by source attr or config override
- WHEN compatibility UI/config fixtures run
- THEN supported suppressions MUST still behave according to the namespace and config contracts.
