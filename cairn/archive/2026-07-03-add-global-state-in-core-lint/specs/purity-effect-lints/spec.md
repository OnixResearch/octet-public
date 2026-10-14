## ADDED Requirements

### Requirement: Global state in core lint

r[octet.purity_effect_lints.global_state_in_core] Octet MUST provide a `global_state_in_core` lint that flags process-wide mutable, lazy, locked, thread-local, or atomic state declarations inside configured core source scopes.

#### Scenario: Core global mutable state is flagged

r[octet.purity_effect_lints.global_state_in_core.flags_global]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it declares `static mut`, thread-local state, lazy initialization state, a global lock, or a global atomic value
- THEN `global_state_in_core` MUST emit a diagnostic at the declaration.
- AND the diagnostic MUST recommend explicit state threading or shell-owned state storage.

#### Scenario: Explicit or immutable state stays clean

r[octet.purity_effect_lints.global_state_in_core.allows_explicit_state]

- GIVEN core code uses `const`, immutable static data without interior mutability, or explicit state parameters and return values
- WHEN the lint runs
- THEN `global_state_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.global_state_in_core.catalog]

- GIVEN `global_state_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
