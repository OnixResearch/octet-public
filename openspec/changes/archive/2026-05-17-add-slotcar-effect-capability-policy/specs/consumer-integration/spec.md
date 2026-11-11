## ADDED Requirements

### Requirement: Consumer Effect Capability Summary [r[consumer-effect-capability-summary]]

The system MUST expose Slotcar effect capability summaries in consumer-facing outputs so downstream teams can triage ambient authority by capability family.

#### Scenario: Consumer output groups findings by capability [r[consumer-effect-capability-summary.scenario.groups-findings]]

- GIVEN a downstream consumer runs Slotcar with JSON or human output
- WHEN findings include mapped effect capability families
- THEN the summary groups counts by capability family
- AND each grouped family can be traced back to the contributing lint names

#### Scenario: Consumer output preserves advisory status [r[consumer-effect-capability-summary.scenario.preserves-advisory-status]]

- GIVEN a capability family is advisory rather than a hard gate
- WHEN the consumer output is generated
- THEN the output preserves that advisory status
- AND it does not turn advisory capability observations into failing exit codes without a separate hard-rule policy
