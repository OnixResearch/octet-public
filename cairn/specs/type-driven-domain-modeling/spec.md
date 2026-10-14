# Type Driven Domain Modeling Specification

## Purpose

Defines the `type-driven-domain-modeling` capability.

## Requirements

### Requirement: Domain identities are strongly typed

r[octet.type_driven_domain_modeling.domain_identities] Octet core APIs SHOULD represent stable domain identities with named Rust types instead of raw strings, path buffers, or numeric primitives when those identities cross non-trivial module boundaries or influence diagnostics, evidence, config, or release decisions.

#### Scenario: Raw identity does not cross a migrated core boundary

r[octet.type_driven_domain_modeling.domain_identities.raw_boundary]

- GIVEN a lint name, lint group, diagnostic identity, evidence family, artifact locator, config key, or policy threshold has been migrated to a typed identity
- WHEN a pure core function receives that value
- THEN the function SHOULD accept the domain type rather than a raw `String`, `&str`, `PathBuf`, or primitive numeric value.
- AND adapters SHOULD remain responsible for parsing or formatting public wire and CLI representations.

#### Scenario: Public representation remains compatible

r[octet.type_driven_domain_modeling.domain_identities.public_compatibility]

- GIVEN an existing consumer uses supported config keys, source attributes, emitted evidence fields, SARIF fields, or CLI output parsing
- WHEN Octet introduces internal domain identity types
- THEN user-facing representations MUST remain compatible unless a separate reviewed migration changes that contract.

### Requirement: Algebraic states replace flag bags

r[octet.type_driven_domain_modeling.algebraic_states] Octet SHOULD model mutually exclusive config, diagnostic, evidence, lifecycle, and gate states with enums or equivalent algebraic Rust types rather than independent booleans or open string tags.

#### Scenario: Invalid state combination is unrepresentable

r[octet.type_driven_domain_modeling.algebraic_states.invalid_combinations]

- GIVEN a state can be clean, warning-only, error-level, skipped, unsupported, or manually reviewed
- WHEN the state is represented in a migrated core
- THEN the representation SHOULD prevent contradictory combinations such as clean and error-level at the same time.
- AND tests SHOULD cover both accepted states and rejected malformed inputs at the adapter boundary.

### Requirement: Refined values validate invariants once

r[octet.type_driven_domain_modeling.refined_values] Octet SHOULD use explicit constructors for refined values whose invariants matter to deterministic behavior, including non-empty collections, canonical names, sorted or deduplicated sets, normalized paths, and named threshold or count values.

#### Scenario: Invalid refined input fails before core use

r[octet.type_driven_domain_modeling.refined_values.negative_inputs]

- GIVEN an input collection is empty where non-empty data is required, a name is non-canonical, a path is outside the accepted form, a set contains duplicates, or a threshold/count value is invalid for its domain
- WHEN the adapter constructs the refined value
- THEN construction MUST fail with a deterministic diagnostic naming the rejected invariant.
- AND the pure core MUST NOT receive the invalid value.

#### Scenario: Refined values remain boring Rust

r[octet.type_driven_domain_modeling.refined_values.boring_rust]

- GIVEN a refined value is added to Octet
- WHEN reviewers inspect its API
- THEN the API SHOULD use explicit Rust constructors, accessors, and pattern matching.
- AND it SHOULD NOT require lens-heavy access, monad terminology, lazy-evaluation assumptions, or type-level machinery unless the design names the concrete bug class prevented.

### Requirement: Domain-type migration is incremental

r[octet.type_driven_domain_modeling.incremental_migration] Domain-type adoption MUST be incremental and reviewable, with each migration naming the core seam, compatibility adapters, positive tests, negative tests, and user-visible non-changes.

#### Scenario: Compatibility adapters are tested

r[octet.type_driven_domain_modeling.incremental_migration.compatibility]

- GIVEN a domain type replaces a raw value inside an Octet core
- WHEN existing Dylint config, source attributes, emitted evidence, SARIF output, artifact manifests, or CLI output pass through the boundary
- THEN adapters MUST preserve accepted inputs and outputs.
- AND malformed inputs MUST fail at the adapter with deterministic diagnostics rather than panicking in the core.
