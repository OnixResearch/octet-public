## ADDED Requirements

### Requirement: Effect Capability Boundary Guidance [r[effect-capability-boundary-guidance]]

The system MUST document a Rust effect-capability boundary model that keeps domain core logic free of ambient IO, time, randomness, env access, process spawning, networking, and remote execution while allowing effects to appear as explicit data or named capability traits.

#### Scenario: Core effects are explicit [r[effect-capability-boundary-guidance-scenario-core-effects-are-explicit]]

- GIVEN a developer designs a core function
- WHEN they consult the guidance
- THEN it describes returning effect descriptions or accepting named capabilities instead of reading ambient state directly

#### Scenario: Shell interprets capabilities [r[effect-capability-boundary-guidance-scenario-shell-interprets-capabilities]]

- GIVEN core logic emits an effect description
- WHEN shell code executes the program
- THEN the shell owns IO, time, randomness, network, process, and remote side effects

### Requirement: Future effect lint classification [r[future-effect-lint-classification]]

The system MUST classify future effect-boundary lint candidates before hard gating them, including effectful globals, effectful constructors, and expanded pure-module checks.

#### Scenario: Lint candidates stay staged [r[future-effect-lint-classification-scenario-lint-candidates-stay-staged]]

- GIVEN an effect-boundary checker has weak static signal
- WHEN policy metadata is reviewed
- THEN the checker remains advisory or human-process-only until positive and negative fixtures prove low false-positive behavior
