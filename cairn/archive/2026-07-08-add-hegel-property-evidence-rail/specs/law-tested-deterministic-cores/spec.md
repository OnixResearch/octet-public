## ADDED Requirements

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
