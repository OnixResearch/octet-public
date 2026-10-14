## ADDED Requirements

### Requirement: Named Capability Vocabulary [r[named-capability-vocabulary]]

The system MUST document a Rust capability vocabulary pattern for domain effects using explicit traits, tokens, or effect enums with names that expose authority and lifecycle obligations.

#### Scenario: Capability name exposes authority [r[named-capability-vocabulary-scenario-capability-name-exposes-authority]]

- GIVEN code needs clock, entropy, credential, object-store, or remote authority
- WHEN the capability is modeled
- THEN its name and API expose the authority being granted rather than hiding it behind generic context objects

#### Scenario: Core admissibility is explicit [r[named-capability-vocabulary-scenario-core-admissibility-is-explicit]]

- GIVEN a capability crosses into core-adjacent logic
- WHEN it is reviewed
- THEN guidance distinguishes pure data capabilities from shell-only effectful capabilities

### Requirement: Deterministic Capability Handlers [r[deterministic-capability-handlers]]

The system MUST require examples for capability traits to include deterministic test handlers such as fixed clocks, seeded entropy, recording stores, or fake remote executors.

#### Scenario: Test handler avoids ambient state [r[deterministic-capability-handlers-scenario-test-handler-avoids-ambient-state]]

- GIVEN a capability example appears in docs
- WHEN the test form is shown
- THEN it uses deterministic handler state rather than ambient time, randomness, network, or filesystem
