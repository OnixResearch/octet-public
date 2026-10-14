## ADDED Requirements

### Requirement: Semantic Review Identity [r[semantic-review-identity]]

The system MUST provide a semantic review identity for fingerprinted Rust items and lint findings that treats names, file paths, and line numbers as metadata rather than the sole identity source. The identity MUST include deterministic hashes for normalized item shape, lint/config context, Slotcar tool version, and dependency identity where available. It MUST NOT include absolute paths, usernames, timestamps, or secrets.

#### Scenario: Normalized item identity survives rename [r[semantic-review-identity.scenario.normalized-item-identity-survives-rename]]

- GIVEN a fingerprinted function is renamed without changing its normalized body shape or dependency identity
- WHEN `cargo slotcar fingerprint --check --output-format json` emits the record
- THEN the semantic identity stays stable while name metadata changes

#### Scenario: Baseline matching uses semantic identity first [r[semantic-review-identity.scenario.baseline-matching-uses-semantic-identity-first]]

- GIVEN a lint finding moves to a new line after a behavior-preserving refactor
- WHEN baseline comparison runs with semantic identity fields present
- THEN the finding is attributed by semantic identity before falling back to path and line

#### Scenario: Dependency identity is explicit when unavailable [r[semantic-review-identity.scenario.dependency-identity-is-explicit-when-unavailable]]

- GIVEN a function calls code whose dependency identity cannot be computed by the source scanner
- WHEN the review manifest is generated
- THEN the manifest records an explicit `dependency_identity_status` instead of silently pretending dependencies were hashed

