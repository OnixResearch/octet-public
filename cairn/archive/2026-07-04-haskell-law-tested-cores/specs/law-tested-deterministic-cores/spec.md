# Law-Tested Deterministic Cores Specification Delta

## ADDED Requirements

### Requirement: Pure cores have named laws

r[octet.law_tested_deterministic_cores.catalog] Octet SHOULD maintain named law coverage for selected pure cores whose deterministic behavior is critical to config, diagnostics, evidence, receipts, artifact identity, boundary scans, or lint classification.

#### Scenario: Config precedence law is named and tested

r[octet.law_tested_deterministic_cores.catalog.config_precedence]

- GIVEN Octet merges defaults, template values, disabled lint lists, and per-lint levels
- WHEN generated valid configurations and concrete examples are evaluated
- THEN tests SHOULD prove the accepted precedence contract, including that disabled lints win over per-lint levels where that contract applies.

#### Scenario: Deterministic identity law is named and tested

r[octet.law_tested_deterministic_cores.catalog.deterministic_identity]

- GIVEN a pure core emits sorted diagnostics, manifests, evidence summaries, receipt identities, or artifact hashes from equivalent normalized inputs
- WHEN the core runs repeatedly
- THEN tests SHOULD prove stable ordering, stable deduplication, and byte-stable output or identity values.

### Requirement: Property tests are reproducible and bounded

r[octet.law_tested_deterministic_cores.reproducible] Property or generated-input law tests MUST be reproducible in the Nix devshell and SHOULD use named case budgets, deterministic runner configuration, and replayable failing inputs.

#### Scenario: Failing generated case can be replayed

r[octet.law_tested_deterministic_cores.reproducible.failure_replay]

- GIVEN a property test finds a failing generated input
- WHEN a maintainer reruns the focused test from the devshell
- THEN the failure SHOULD be reproducible from saved framework output or a checked-in regression fixture.
- AND the failure report SHOULD name the law that was violated.

### Requirement: Law suites include negative behavior

r[octet.law_tested_deterministic_cores.negative_cases] Every law suite SHOULD include negative or adversarial cases that verify malformed input, contradictory state, duplicate identity, unsupported values, and invalid classifier facts fail correctly.

#### Scenario: Invalid generated input fails closed

r[octet.law_tested_deterministic_cores.negative_cases.fail_closed]

- GIVEN generated or concrete inputs contain malformed config, contradictory states, duplicate identities, unsupported discriminants, invalid paths, or impossible classifier facts
- WHEN the pure core or adapter validates those inputs
- THEN it MUST return a typed error or deterministic diagnostic rather than panic, silently accept the input, or emit unstable output.

### Requirement: Law tests complement fixture rails

r[octet.law_tested_deterministic_cores.fixture_complement] Property and law tests MUST complement, not replace, UI fixtures, snapshot tests, CLI golden output, transcript evidence, and artifact verification required for user-visible behavior.

#### Scenario: User-visible behavior remains fixture-backed

r[octet.law_tested_deterministic_cores.fixture_complement.user_visible]

- GIVEN a law suite covers a pure classifier, config merge, or evidence transform
- WHEN the behavior produces user-visible diagnostics, CLI output, source-level lint matches, or persisted artifacts
- THEN maintainers SHOULD keep positive and negative fixture or golden-output coverage for the user-visible contract.
