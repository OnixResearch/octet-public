# Law Tested Deterministic Cores Specification

## Purpose

Defines the `law-tested-deterministic-cores` capability.

## Requirements

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

### Requirement: Hegel-backed deterministic core laws
r[octet.law_tested_deterministic_cores.hegel] Octet SHOULD use Hegel RS property tests for selected pure deterministic cores when generated inputs materially improve coverage beyond concrete fixtures.

#### Scenario: Hegel laws target pure cores
r[octet.law_tested_deterministic_cores.hegel.pure_core]
- GIVEN a pure Octet core handles config merging, diagnostics ordering, artifact verification, fingerprint identity, proof-ref validation, migration manifests, standards policies, or evidence transforms
- WHEN maintainers add a Hegel property law for that core
- THEN the law SHOULD call deterministic in-memory functions and MUST NOT require filesystem, process, network, clock, or environment effects inside the core under test.

#### Scenario: Hegel law settings are reproducible
r[octet.law_tested_deterministic_cores.hegel.replayable]
- GIVEN a Hegel law is part of Octet validation
- WHEN the law runs in the Nix devshell or CI
- THEN it SHOULD use named case budgets, deterministic or replayable settings, and failure output that lets maintainers reproduce or preserve a minimal failing case.

#### Scenario: Hegel laws include negative generated cases
r[octet.law_tested_deterministic_cores.hegel.negative]
- GIVEN generated inputs contain malformed config, contradictory state, duplicate identity, unsupported discriminants, stale hashes, or impossible classifier facts
- WHEN a Hegel law evaluates the pure core
- THEN Octet MUST return typed errors or deterministic diagnostics rather than panic, silently accept invalid input, or emit unstable output.

#### Scenario: Hegel complements fixture rails
r[octet.law_tested_deterministic_cores.hegel.fixture_complement]
- GIVEN a Hegel law covers a pure classifier, config merge, evidence transform, or artifact identity core
- WHEN the behavior produces user-visible diagnostics, CLI output, source-level lint matches, or persisted artifacts
- THEN maintainers SHOULD keep positive and negative UI, golden, transcript, or artifact-verification fixtures for the user-visible contract.

### Requirement: Hegel laws use a checked value-based catalog

r[octet.law_tested_deterministic_cores.hegel_catalog]

Octet MUST maintain a checked catalog for selected Hegel laws that binds each pure core, law ID, generator, positive property, negative property, budget, replay method, and fixture complement.

#### Scenario: Selected core enters the catalog

- GIVEN generated inputs materially improve coverage for a deterministic in-memory core
- WHEN maintainers add a Hegel law
- THEN the catalog MUST name the generated-input benefit, positive property, negative property, budget, replay method, and complementary fixture
- AND the property body MUST NOT require filesystem, process, network, clock, or environment effects

#### Scenario: Law duplicates concrete coverage

- GIVEN a proposed Hegel law adds no generated-input benefit beyond existing concrete fixtures
- WHEN catalog validation runs
- THEN validation MUST reject the entry or require a documented diagnostic-only exemption

#### Scenario: Counterexample becomes a regression case

- GIVEN a cataloged Hegel law finds a minimal failing input
- WHEN maintainers accept the fix
- THEN the failure MUST remain replayable through stable Hegel output or a checked-in positive or negative regression fixture

#### Scenario: User-visible behavior remains covered

- GIVEN a cataloged law covers a core that affects diagnostics, CLI output, persisted artifacts, or source-level findings
- WHEN law coverage is reviewed
- THEN matching positive and negative UI, golden, transcript, or artifact fixtures MUST remain in place
