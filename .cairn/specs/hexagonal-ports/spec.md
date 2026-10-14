# Hexagonal Ports Specification

## Purpose

Defines the `hexagonal-ports` capability.

## Requirements

### Requirement: Port mechanism is opt-in

r[octet.hexagonal_ports.opt_in] Project port checks MUST run only through an activated project architecture policy with declared port mechanisms.

#### Scenario: No project policy means no port evaluation

r[octet.hexagonal_ports.opt_in.noop]

- GIVEN a project declares no architecture-policy pointer
- WHEN `cargo octet check` runs
- THEN Octet MUST emit no project port diagnostics or port-pass claim
- AND it MUST NOT infer ports from undeclared traits

#### Scenario: Active policy has no declared ports

- GIVEN a valid active project policy declares no port mechanism
- WHEN project port checks resolve
- THEN Octet MUST emit zero port diagnostics
- AND it MUST record that no port obligations were selected

### Requirement: Port mechanism contract

r[octet.hexagonal_ports.mechanism] Project policy MUST declare port trait identities, one application-owner scope, permitted adapter scopes, forbidden concrete types, and reserved status.

#### Scenario: Port mechanism is well-formed

r[octet.hexagonal_ports.mechanism.well_formed]

- GIVEN a project declares port traits with `application_owner_scope` and permitted adapter scopes
- WHEN Nickel validates the project policy
- THEN validation MUST succeed when all scopes resolve to permitted roles

#### Scenario: Legacy core owner is declared

- GIVEN a project declares `core_owner_scope` or assigns port ownership to `domain-core`
- WHEN policy validation runs
- THEN validation MUST fail with a deterministic application-owner migration diagnostic

### Requirement: Port invariants

r[octet.hexagonal_ports.invariants] The checker MUST verify that ports are application-owned, adapters implement them, and concrete implementations remain outside domain and application scopes.

#### Scenario: Port defined in application scope

r[octet.hexagonal_ports.invariants.defined_in_core]

- GIVEN a port is declared but no compiler-visible trait exists in its application-owner scope
- WHEN the port check runs
- THEN it MUST emit a diagnostic that names the missing application-owned port

#### Scenario: Non-reserved port has an adapter implementation

r[octet.hexagonal_ports.invariants.adapter_impl]

- GIVEN a non-reserved port has no compiler-visible implementation in a permitted adapter scope
- WHEN the port check runs
- THEN it MUST emit a diagnostic naming the port and permitted adapter scopes
- AND a reserved port MUST remain visible without a missing-implementation finding

#### Scenario: Core or application has concrete implementation

r[octet.hexagonal_ports.invariants.no_concrete_impl]

- GIVEN a domain-core or application-shell scope implements a port for a concrete infrastructure type
- WHEN the port check runs
- THEN it MUST emit a placement diagnostic naming the port, concrete type, role, and source location
- AND remediation MUST move the implementation to a permitted adapter scope
