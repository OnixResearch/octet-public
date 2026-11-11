## ADDED Requirements

### Requirement: Slotcar Effect Capability Vocabulary [r[slotcar-effect-capability-vocabulary]]

The system MUST maintain a stable Slotcar capability vocabulary for effect-like authority that includes time, randomness, filesystem, network, process spawning, environment variables, secrets, remote execution, unsafe code, global state, and concurrency.

#### Scenario: Policy registry names capability family [r[slotcar-effect-capability-vocabulary.scenario.registry-names-family]]

- GIVEN a policy or lint detects an ambient effect or shell authority
- WHEN the policy registry is exported
- THEN the entry includes a stable Slotcar capability family name
- AND the family name belongs to the approved capability vocabulary

#### Scenario: FCIS guidance classifies core and shell authority [r[slotcar-effect-capability-vocabulary.scenario.fcis-classifies-authority]]

- GIVEN a policy describes deterministic functional core / imperative shell boundaries
- WHEN a reader reviews the capability vocabulary
- THEN ambient time, randomness, filesystem, network, process, environment, secrets, remote execution, unsafe, global state, and concurrency authority are classified as core-prohibited unless represented as explicit data or named capability traits
- AND shell-allowed uses are documented as interpreter responsibilities

### Requirement: Slotcar Effect Capability Mapping Validation [r[slotcar-effect-capability-mapping-validation]]

The system MUST validate that effect-like lint and policy outputs either map to a known capability family or explicitly report an unknown capability status. Unknown status MUST be visible in human and machine-readable summaries.

#### Scenario: Known capability appears in output [r[slotcar-effect-capability-mapping-validation.scenario.known-capability-output]]

- GIVEN a lint finding is mapped to the `time` or `filesystem` capability family
- WHEN JSON, SARIF, or human summary output is generated
- THEN the output includes the mapped capability family
- AND effect summaries count the finding under that family

#### Scenario: Unknown capability is visible [r[slotcar-effect-capability-mapping-validation.scenario.unknown-capability-visible]]

- GIVEN a new lint has no capability-family mapping
- WHEN policy validation or summary output runs
- THEN the result reports an explicit unknown capability status
- AND the unknown status is not omitted from effect summaries
