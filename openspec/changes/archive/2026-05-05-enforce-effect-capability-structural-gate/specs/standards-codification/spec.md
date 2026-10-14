## ADDED Requirements

### Requirement: Artifact-backed effect capability mechanism obligations [r[artifact-backed-effect-capability-mechanism-obligations]]

The system MUST let the `tigerstyle.architecture.effect-capability-boundaries` policy export capability-api mechanism obligations through generated policy artifacts.

#### Scenario: Capability obligations are exported [r[artifact-backed-effect-capability-mechanism-obligations.scenario.exported]]

- GIVEN the Nickel registry declares a capability-api mechanism for effect capability boundaries
- WHEN generated policy artifacts are exported
- THEN the `tigerstyle.architecture.effect-capability-boundaries` artifact entry carries forbidden ambient APIs, core source scopes, shell constructor scopes, and approved capability names
- AND the artifact-backed checker can consume those fields without reading Nickel source directly

### Requirement: Core ambient authority structural rejection [r[core-ambient-authority-structural-rejection]]

The system MUST reject declared core source code that performs configured ambient authority operations instead of accepting injected data or narrow capability traits.

#### Scenario: Core ambient operation fails [r[core-ambient-authority-structural-rejection.scenario.core-ambient-fails]]

- GIVEN a Tigerstyle architecture fixture whose declared core scope calls a configured ambient authority API such as `std::fs`, `std::env`, `std::net`, `SystemTime::now`, or process spawning
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the architecture family
- THEN the report contains a structural diagnostic for `tigerstyle.architecture.effect-capability-boundaries`
- AND the diagnostic remediation tells the developer to move ambient constructors to the shell and inject data or named capabilities into core

### Requirement: Injected capability and shell construction acceptance [r[injected-capability-shell-construction-acceptance]]

The system MUST accept declared core source code that represents effects as data or named capability traits while allowing ambient constructors in declared shell construction scopes.

#### Scenario: Injected core capability passes [r[injected-capability-shell-construction-acceptance.scenario.injected-core-passes]]

- GIVEN a Tigerstyle architecture fixture whose core scope accepts a named capability trait or effect data instead of reading ambient state directly
- WHEN the artifact-backed architecture check runs
- THEN no `tigerstyle.architecture.effect-capability-boundaries` diagnostic is emitted for the core fixture

#### Scenario: Shell constructor scope passes [r[injected-capability-shell-construction-acceptance.scenario.shell-scope-passes]]

- GIVEN a Tigerstyle architecture fixture whose declared shell scope owns the configured ambient constructor
- WHEN the artifact-backed architecture check runs
- THEN no `tigerstyle.architecture.effect-capability-boundaries` diagnostic is emitted for that shell fixture
