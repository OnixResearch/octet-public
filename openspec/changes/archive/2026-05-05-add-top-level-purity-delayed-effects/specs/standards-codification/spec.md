## ADDED Requirements

### Requirement: Top Level Purity Guidance [r[top-level-purity-guidance]]

The system MUST document that Rust globals, static initializers, lazy initializers, constructors, Default implementations, and builders should not hide filesystem, env, clock, randomness, network, process, or spawn effects.

#### Scenario: Initializer side effects are rejected by guidance [r[top-level-purity-guidance-scenario-initializer-side-effects-are-rejected-by-guidance]]

- GIVEN a LazyLock initializer reads env or files
- WHEN a reviewer applies the guidance
- THEN it is classified as shell work or delayed effect construction rather than pure initialization

#### Scenario: Constructors validate only [r[top-level-purity-guidance-scenario-constructors-validate-only]]

- GIVEN a `new`, `default`, or `build` function exists in core logic
- WHEN it is reviewed
- THEN the accepted shape validates and constructs values without performing ambient effects

### Requirement: Delayed Effect Pattern [r[delayed-effect-pattern]]

The system MUST document delayed-effect alternatives such as returning commands, plans, tasks, or capability requests for shell interpretation.

#### Scenario: Effect is represented as data [r[delayed-effect-pattern-scenario-effect-is-represented-as-data]]

- GIVEN construction needs future IO
- WHEN the code follows the delayed-effect pattern
- THEN it returns an explicit effect description and leaves execution to shell code
