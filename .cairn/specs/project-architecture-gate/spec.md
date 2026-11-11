# Project Architecture Gate Specification

## Purpose

Defines the `project-architecture-gate` capability.

## Requirements

### Requirement: Cargo check integrates the activated architecture rail

r[octet.project_architecture_gate.orchestration]

`cargo octet check` MUST run project architecture evaluation when project metadata selects a valid architecture policy.

#### Scenario: Configured project runs all selected phases

- GIVEN a project selects an active architecture policy
- WHEN `cargo octet check` runs
- THEN the runner MUST run lint, compiler-fact, Cargo-graph, and architecture phases in deterministic order
- AND `admission` mode MUST run effect closure only after the architecture gate passes

#### Scenario: Rail statuses stay separate

- GIVEN one selected phase passes and another phase fails
- WHEN the parent result is built
- THEN each phase MUST retain its own status, evidence tier, findings, and non-claims
- AND one passing phase MUST NOT mask or promote another phase

### Requirement: Activated evaluation fails on missing inputs

r[octet.project_architecture_gate.input_integrity]

An active architecture run MUST treat unavailable or mismatched policy, Cargo, compiler, scope, target, and feature inputs as integration failures.

#### Scenario: Cargo metadata fails

- GIVEN locked Cargo metadata cannot execute, parse, or match the selected workspace
- WHEN architecture evaluation starts
- THEN the runner MUST fail with a Cargo-metadata integration diagnostic
- AND it MUST NOT return architecture-clean

#### Scenario: Compiler coverage is incomplete

- GIVEN one declared production target or scope lacks a valid Architecture IR shard
- WHEN architecture evaluation starts
- THEN the runner MUST fail with an incomplete-coverage diagnostic before policy acceptance

### Requirement: Dependency direction uses project roles

r[octet.project_architecture_gate.role_graph]

The gate MUST evaluate direct and configured transitive dependency edges against project roles and exact provider classifications.

#### Scenario: Inward role graph passes

- GIVEN domain cores depend only on declared core or pure providers, adapters depend on application contracts, and a composition root depends on shell and adapters
- WHEN dependency evaluation runs
- THEN the role graph MUST pass

#### Scenario: Core or shell selects infrastructure

- GIVEN a domain core or application shell depends on or constructs a concrete adapter or infrastructure provider
- WHEN dependency evaluation runs
- THEN the gate MUST emit a forbidden outward-edge or construction diagnostic

#### Scenario: Adapter construction is wrapped in a result

- GIVEN a domain core or application shell calls an adapter constructor returning `Result<Self, _>`, `Option<_>`, `Box<_>`, or `Arc<_>`
- WHEN construction evaluation runs
- THEN the gate MUST emit the construction diagnostic for the adapter type held by value in the result

#### Scenario: Adapter use is not adapter construction

- GIVEN an adapter or shell borrows, locks, pins, iterates, moves, or clones a shared handle to an injected adapter, or calls a generic function that returns an adapter value it received
- WHEN construction evaluation runs
- THEN the gate MUST NOT emit a construction diagnostic for that adapter type

#### Scenario: Composition root selects adapters

- GIVEN a declared composition root constructs permitted inbound and outbound adapters and supplies them to the application shell
- WHEN dependency evaluation runs
- THEN that selection MUST be accepted
- AND the composition root MUST NOT acquire domain-policy ownership from this acceptance

### Requirement: Gate evaluates FCIS and Hexagonal invariants

r[octet.project_architecture_gate.fcis]

The gate MUST evaluate complete roles, domain-core purity, application-owned ports, adapter placement, external-type leakage, capability classification, and optional host-independence obligations.

#### Scenario: Functional core and imperative shell are separated

- GIVEN domain-core functions use explicit facts and return decisions, state, events, or effect plans without direct effects
- WHEN FCIS evaluation runs
- THEN the domain-core obligations MUST pass

#### Scenario: Domain core performs or hides an effect

- GIVEN a direct or selected locally transitive domain-core call performs time, randomness, filesystem, network, process, environment, observability, spawn, synchronization, resource, or other forbidden work
- WHEN FCIS evaluation runs
- THEN strict modes MUST block with the exact effect and call evidence

#### Scenario: Port and adapter placement is valid

- GIVEN an application-owned port has compiler-resolved adapter implementations only in permitted adapter roles
- WHEN Hexagonal evaluation runs
- THEN the port obligation MUST pass

#### Scenario: Infrastructure type leaks inward

- GIVEN a domain or application contract exposes a compiler-resolved transport, storage, framework, or vendor type forbidden by policy
- WHEN boundary evaluation runs
- THEN strict modes MUST block with the exact signature and type origin

### Requirement: Architecture modes control finding outcomes

r[octet.project_architecture_gate.modes]

The gate MUST preserve all architecture findings while applying the selected mode to promotion status.

#### Scenario: Inventory or advisory finds a violation

- GIVEN a complete active run selects `inventory` or `advisory`
- WHEN an architecture rule fails
- THEN output MUST retain the finding and report that architecture is not clean
- AND the finding alone MUST NOT produce a strict gate failure

#### Scenario: Gate finds a violation or required unknown

- GIVEN a complete active run selects `gate`
- WHEN a hard architecture rule fails or a required compiler fact is unknown
- THEN the architecture phase MUST block

### Requirement: Admission composes effect closure

r[octet.project_architecture_gate.admission]

`admission` mode MUST require a successful architecture gate and successful effect closure over declared domain-core scopes.

#### Scenario: Gate and closure pass

- GIVEN architecture evaluation passes and the existing effect-closure rail admits every declared core scope
- WHEN admission completes
- THEN the parent result MAY report architecture admission
- AND it MUST retain the effect-closure receipt as separate identified evidence

#### Scenario: Gate or closure fails

- GIVEN architecture evaluation or effect closure blocks
- WHEN admission completes
- THEN the parent result MUST block and identify the failed phase

### Requirement: Architecture waivers remain exact

r[octet.project_architecture_gate.waivers]

The gate MUST apply only exact current architecture waivers after stable finding IDs are computed.
Finding IDs, compiler unit identities, and the waiver policy identity MUST NOT depend on the policy's waiver list, so adding a waiver cannot make itself stale.

#### Scenario: Exact waiver marks one finding

- GIVEN one waiver matches one current finding, policy, source identity, owner, reason, and review condition
- WHEN waiver evaluation runs
- THEN the finding MAY become `waived`
- AND the architecture result MUST distinguish `waived` from `clean`

#### Scenario: Rust lint allow targets architecture finding

- GIVEN source uses `#[allow]` for a project architecture finding
- WHEN architecture evaluation runs
- THEN the Rust attribute MUST NOT suppress the project architecture finding

### Requirement: Architecture receipt binds the complete evaluation

r[octet.project_architecture_gate.evidence]

An active run MUST emit a Rust-owned, versioned, BLAKE3-addressed architecture receipt for every complete architecture evaluation attempt.

#### Scenario: Receipt is complete and repeatable

- GIVEN unchanged policy, command, Cargo graph, Architecture IR, coverage, roles, findings, waivers, target, and toolchain
- WHEN evaluation runs twice
- THEN canonical receipt bytes and identity MUST match
- AND the receipt MUST include explicit source-shape, compiler, dependency, runtime, correctness, and release non-claims

#### Scenario: Stored evidence is modified

- GIVEN the receipt or any bound policy, graph, IR, coverage, status, waiver, or parent artifact changes
- WHEN artifact verification runs
- THEN verification MUST fail with an identity or linkage diagnostic

### Requirement: Projects without policy remain compatible

r[octet.project_architecture_gate.compatibility]

The integrated architecture phase MUST remain inactive for projects without an architecture-policy pointer.

#### Scenario: Existing consumer runs normal check

- GIVEN an existing consumer has no architecture-policy pointer
- WHEN `cargo octet check` runs after this change
- THEN command behavior, child environment, lint findings, exit status, and existing artifacts MUST remain compatible

#### Scenario: Output avoids complete proof claims

- GIVEN a configured architecture run passes
- WHEN human or machine output describes the result
- THEN output MUST scope success to the selected policy, targets, features, compiler facts, Cargo graph, and effect evidence
- AND it MUST NOT claim complete FCIS proof, runtime determinism, whole-program correctness, or release eligibility

### Requirement: Integrated gate verification is adversarial

r[octet.project_architecture_gate.verification]

The implementation MUST include positive and negative fixtures for orchestration, input integrity, roles, FCIS, ports, modes, waivers, evidence, and compatibility.

#### Scenario: Bypass fixture matrix runs

- GIVEN fixtures narrow scope, mimic traits in comments, alias vendor types, hide effects in helpers, use macros, omit targets, disable adapters, or leave dispatch unresolved
- WHEN integrated validation runs
- THEN strict positive fixtures MUST pass deterministically
- AND each bypass fixture MUST block with its declared diagnostic class

### Requirement: Authorization identities follow protected effect declarations
r[octet.project_architecture_gate.authority_scoped_receipt_ids]

Receipt verification MUST require nonempty authorization fact and type identity arrays when at least one outbound-effect classification is protected.
An unprotected-only effect list MUST NOT require protected authorization identities.
The authority discriminator MUST control this requirement, independently of declaration order and optional protocol payload presence.
Receipt identity, schema, evidence tier, parent bindings, required non-claims, and architecture findings MUST retain their existing checks.
Successful self-verification MUST NOT promote an unprotected effect to authorized execution or establish a passing architecture phase.

#### Scenario: Unprotected receipt has no protected identities

- GIVEN a complete valid architecture evaluation with only unprotected outbound effects
- WHEN the producer emits empty authorization identity arrays and the receipt is serialized and read back
- THEN receipt self-verification MUST succeed and preserve the complete classification and non-claims

#### Scenario: Protected requirement survives a mixed effect list

- GIVEN a protected or mixed effect list with a current receipt identity
- WHEN either authorization identity array is empty
- THEN verification MUST reject the receipt regardless of declaration order or optional protocol payload presence

#### Scenario: Unprotected classification does not bypass other checks

- GIVEN separate cases with a tampered receipt, a missing required non-claim, or an actual unclassified outbound effect
- WHEN receipt verification or architecture evaluation checks the affected boundary
- THEN the corresponding identity, claim-boundary, or effect-classification check MUST reject that case

### Requirement: Integration-test roles are compiler-bound
r[octet.project_architecture_gate.test_ownership]

A selected independent Cargo integration-test unit MAY use an explicit matching test source role.
Its source root, target kind, configuration, and membership MUST be bound to compiler selection and checked during artifact replay.
Library and binary test configurations MUST retain their production roles.
Missing roots, test-looking names, and conflicting production source roles MUST remain blocking.
Test-only observations MUST remain covered and explicitly disposed as non-production evidence.
They MUST NOT discharge production architecture obligations. Shared production observations MUST NOT be hidden.
The full selected lint and compilation scope MUST remain enabled.
Production evidence MUST retain valid references after partitioning; missing test-only subjects MUST NOT become skipped production checks.
The receipt MUST bind its test-only disposition and the unchanged full-input identities. Artifact replay MUST compare the complete disposition.
No-std evidence MUST come from the active compiler-parsed crate attribute and retain each observing unit's membership.

#### Scenario: Independent tests coexist with production units

- GIVEN explicit integration-test source roles and complete compiler-bound production and test observations
- WHEN the gate evaluates production architecture
- THEN integration tests MUST remain covered with separate dispositions and production findings MUST remain visible

#### Scenario: A production unit imports test-labelled source

- GIVEN a production library, binary, or their test configuration imports test-labelled source
- WHEN ownership is checked
- THEN the source-role conflict MUST block regardless of filename or unit ordering

#### Scenario: Test evidence attempts to satisfy production obligations

- GIVEN test-only witnesses, ports, no-std facts, or unknowns and shared production observations
- WHEN architecture evaluation and artifact replay run
- THEN test-only evidence MUST NOT satisfy production obligations or hide required production unknowns

#### Scenario: A resealed receipt changes its test-only disposition

- GIVEN unchanged compiler inputs and a changed disposition with refreshed receipt, phase-status, and provenance hashes
- WHEN the full artifact bundle is replayed
- THEN reevaluation MUST reject the changed disposition even if receipt self-verification succeeds

#### Scenario: A production import uses a parent-relative path or source alias

- GIVEN test-labelled source imported through a parent-relative path or source symlink
- WHEN source ownership is established
- THEN physical workspace ownership MUST remain visible or collection MUST reject before emitting an accepted architecture result

#### Scenario: Crate configuration differs between production and test observations

- GIVEN active, inactive, library-test-disabled, or integration-only no-std attributes
- WHEN compiler configuration evidence is collected
- THEN only units with the active parsed crate attribute MUST receive no-std facts, and test-only facts MUST NOT satisfy production obligations

#### Scenario: A package-scoped no-std obligation binds library units

- GIVEN a `core_no_std` obligation that names a domain-core package with a build script, binary, example, or bench target
- WHEN the obligation is evaluated
- THEN the package's library and library-test units MUST each carry the active no-std fact, and its other targets MUST NOT be required to carry it
