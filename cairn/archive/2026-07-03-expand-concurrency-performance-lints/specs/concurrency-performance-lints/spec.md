## ADDED Requirements

### Requirement: Candidate ranking matrix

r[octet.concurrency_performance_lints.candidate_matrix] Octet MUST rank new concurrency and performance lint candidates by signal strength, false-positive risk, type-information needs, fixture complexity, default-level recommendation, and remediation clarity before implementation.

#### Scenario: Candidate matrix is reviewable

r[octet.concurrency_performance_lints.candidate_matrix.reviewable]

- GIVEN maintainers propose new concurrency or performance lints
- WHEN the candidate matrix is rendered
- THEN each candidate MUST include a score or qualitative rating for signal, noise, type-info dependency, fixture complexity, default level, and remediation.
- AND the matrix MUST identify which candidates are accepted, deferred, or rejected for this change.

### Requirement: First-wave concurrency and performance lints

r[octet.concurrency_performance_lints.first_wave] New concurrency and performance lints MUST be narrow, fixture-backed, and scoped before they enter the default lint catalog.

#### Scenario: Async/concurrency lifecycle lint has guardrails

r[octet.concurrency_performance_lints.first_wave.async_guardrails]

- GIVEN a first-wave concurrency lint targets an async lifecycle hazard such as holding a lock across `.await`, blocking in async, ignored task handles, or unbounded task spawning
- WHEN the lint ships
- THEN it MUST include positive fixtures for the hazard and negative fixtures for clean lifecycle handling.
- AND the lint metadata MUST state its default level and remediation.

#### Scenario: Performance lint requires scope or clear local shape

r[octet.concurrency_performance_lints.first_wave.performance_scope]

- GIVEN a first-wave performance lint targets allocation, cloning, or hot-loop behavior
- WHEN the lint ships
- THEN it SHOULD require a declared hot scope or a clear local syntactic shape unless evidence justifies a broader default.

#### Scenario: Catalog freshness includes new domains

r[octet.concurrency_performance_lints.first_wave.catalog]

- GIVEN a first-wave lint is added to `src/lib.rs`
- WHEN catalog freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, and default-level snapshots MUST include the new lint identity.
