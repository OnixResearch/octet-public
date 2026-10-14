## ADDED Requirements

### Requirement: Interior mutability in core lint

r[octet.purity_effect_lints.interior_mutability_in_core] Octet MUST provide an `interior_mutability_in_core` lint that flags interior-mutability cells, locks, atomics, or unsafe cells used as fields or direct construction sites inside configured core source scopes.

#### Scenario: Core interior mutability is flagged

r[octet.purity_effect_lints.interior_mutability_in_core.flags_interior_mutability]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it declares a field or direct construction site using `Cell`, `RefCell`, `UnsafeCell`, a lock type, or an atomic type
- THEN `interior_mutability_in_core` MUST emit a diagnostic at the type or constructor.
- AND the diagnostic MUST recommend explicit state-transition values or shell-owned synchronization.

#### Scenario: Explicit immutable state stays clean

r[octet.purity_effect_lints.interior_mutability_in_core.allows_explicit_state]

- GIVEN core code uses immutable fields, receives and returns explicit state values, or shell code owns synchronization primitives
- WHEN the lint runs
- THEN `interior_mutability_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.interior_mutability_in_core.catalog]

- GIVEN `interior_mutability_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
