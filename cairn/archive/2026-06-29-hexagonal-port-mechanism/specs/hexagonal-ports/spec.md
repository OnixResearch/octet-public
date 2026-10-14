## ADDED Requirements

### Requirement: Port mechanism is opt-in

r[octet.hexagonal_ports.opt_in] The port check MUST be opt-in: a consumer that declares no `port` mechanism MUST receive zero port diagnostics.

#### Scenario: No port mechanism means no port diagnostics

r[octet.hexagonal_ports.opt_in.noop]

- GIVEN a consumer declares no `port` mechanism
- WHEN the architecture checker runs the port checks
- THEN it MUST emit zero port diagnostics.
- AND it MUST NOT treat an undeclared `trait` as a port.

### Requirement: Port mechanism contract

r[octet.hexagonal_ports.mechanism] The policy registry MUST accept a `port` mechanism declaring port trait names, the core owner scope, permitted adapter scopes, forbidden concrete types, and a reserved flag.

#### Scenario: Port mechanism is well-formed

r[octet.hexagonal_ports.mechanism.well_formed]

- GIVEN a consumer declares a `port` mechanism with `port_traits`, `core_owner_scope`, and `permitted_adapter_scopes`
- WHEN Nickel evaluates the policy contract
- THEN evaluation MUST succeed.
- AND a `port` mechanism omitting `core_owner_scope` MUST fail closed.

### Requirement: Port invariants

r[octet.hexagonal_ports.invariants] The checker MUST verify that ports are defined in core, that each non-reserved port has an adapter implementation, and that core never implements a port for a concrete infrastructure type.

#### Scenario: Port defined in core

r[octet.hexagonal_ports.invariants.defined_in_core]

- GIVEN a port trait is declared but not found in the declared `core_owner_scope`
- WHEN the port check runs
- THEN it MUST emit a diagnostic stating the port must be declared in core.

#### Scenario: Non-reserved port has an adapter impl

r[octet.hexagonal_ports.invariants.adapter_impl]

- GIVEN a non-reserved port has no `impl` in any permitted adapter scope
- WHEN the port check runs
- THEN it MUST emit a diagnostic naming the port and the permitted adapter scopes.
- AND a port flagged `reserved: true` MUST NOT emit the missing-impl diagnostic.

#### Scenario: Core has no concrete-infrastructure port impl

r[octet.hexagonal_ports.invariants.no_concrete_impl]

- GIVEN the declared core scope contains an `impl <Port> for <ConcreteInfraType>`
- WHEN the port check runs
- THEN it MUST emit a diagnostic stating core must keep ports abstract and the concrete impl belongs in an adapter.
- AND the diagnostic MUST name the port, the concrete type, and the core source location.
