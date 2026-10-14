## ADDED Requirements

### Requirement: Dependency-direction graph check

r[octet.dependency_direction.invariants] The architecture checker MUST verify workspace dependency direction against the declared core, shell, and adapter crate sets using `cargo metadata`.

#### Scenario: Core never depends on adapters

r[octet.dependency_direction.invariants.core_to_adapter]

- GIVEN a core crate's manifest or transitive resolve reaches an adapter crate
- WHEN the dependency-direction check runs
- THEN it MUST emit a diagnostic naming the dependent core crate, the adapter dependency, and whether the edge is direct or transitive.
- AND the remediation MUST point at moving the dependency to an adapter or injecting a port.

#### Scenario: Adapters depend inward on core

r[octet.dependency_direction.invariants.adapter_inward]

- GIVEN an adapter crate does not depend on a core crate
- WHEN the dependency-direction check runs
- THEN it MUST emit a diagnostic stating the adapter must depend inward on a declared core crate.

#### Scenario: Core depends only on core plus std

r[octet.dependency_direction.invariants.core_isolation]

- GIVEN a core crate depends on an external infrastructure crate that is not std and not another core crate
- WHEN the dependency-direction check runs
- THEN it MUST emit a diagnostic flagging the external infrastructure dependency in core.

### Requirement: Direct and transitive edge classification

r[octet.dependency_direction.edges] The check MUST distinguish direct dependency edges from transitive leakage and report them as distinct diagnostics.

#### Scenario: Transitive leakage is reported distinctly

r[octet.dependency_direction.edges.transitive]

- GIVEN a core crate reaches an adapter crate only through a non-core intermediate
- WHEN the dependency-direction check runs
- THEN the diagnostic MUST label the edge as transitive and MUST name the intermediate crate.

### Requirement: Configurable dependency-edge scope

r[octet.dependency_direction.scope] The boundary config MUST let consumers choose which dependency kinds the direction check inspects.

#### Scenario: Dev-dependencies excluded by default

r[octet.dependency_direction.scope.default]

- GIVEN a consumer declares no explicit dependency-kind scope
- WHEN the check resolves the edge scope
- THEN it MUST inspect normal and build dependencies and MUST exclude dev-dependencies.
- AND a consumer that opts in to dev-dependency inspection MUST be able to enable it through the boundary field.

### Requirement: Pure core and imperative shell boundary

r[octet.dependency_direction.shell] The edge classification MUST be a pure function in core and the `cargo metadata` execution MUST stay in the imperative shell.

#### Scenario: Cargo metadata execution stays out of core

r[octet.dependency_direction.shell.purity]

- GIVEN the dependency-direction check runs
- WHEN the pure core classifies edges
- THEN the core function MUST accept an already-parsed graph and MUST NOT execute cargo, read the filesystem, or touch the environment.
- AND the imperative shell MUST own `cargo metadata` invocation and package classification.
