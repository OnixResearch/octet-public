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
