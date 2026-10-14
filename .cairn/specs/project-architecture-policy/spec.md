# Project Architecture Policy Specification

## Purpose

Defines the `project-architecture-policy` capability.

## Requirements

### Requirement: Project architecture activation is explicit

r[octet.project_architecture_policy.activation]

Octet MUST activate project FCIS and Hexagonal architecture evaluation only when `[workspace.metadata.octet].architecture_policy` names a valid workspace-relative policy.

#### Scenario: Project has no architecture pointer

- GIVEN a workspace has no `[workspace.metadata.octet].architecture_policy` value
- WHEN `cargo octet check` resolves project configuration
- THEN Octet MUST NOT load a project architecture policy or emit project architecture findings or receipts
- AND Octet MUST NOT claim that the project passed FCIS or Hexagonal architecture checks

#### Scenario: Project selects a valid architecture policy

- GIVEN Cargo metadata points to a workspace-relative `octet-architecture.ncl` with a current deterministic export
- WHEN project configuration resolves
- THEN Octet MUST activate the declared architecture mode and bind the policy and export identities

#### Scenario: Architecture pointer escapes or is unreadable

- GIVEN the pointer is absolute, escapes the workspace, names a missing file, or lacks a current deterministic export
- WHEN project configuration resolves
- THEN the active architecture request MUST fail closed before architecture evaluation

### Requirement: Project architecture policy is typed and deterministic

r[octet.project_architecture_policy.schema]

Octet MUST provide a typed Nickel contract for project identity, schema, mode, production scope, targets, roles, dependency rules, ports, capabilities, waivers, limits, and non-claims.

#### Scenario: Complete policy exports deterministically

- GIVEN a project policy satisfies the current Nickel contract
- WHEN policy validation and export run twice under unchanged inputs
- THEN both runtime exports MUST be byte-identical
- AND the generated manifest MUST bind the Nickel source, contract dependencies, output, and evaluator identity with BLAKE3

#### Scenario: Policy field is missing or unknown

- GIVEN a policy omits a required field or names an unknown schema, mode, role, scope kind, capability family, or waiver field
- WHEN Nickel validation runs
- THEN validation MUST fail with a deterministic field diagnostic

### Requirement: Production roles are explicit and complete

r[octet.project_architecture_policy.roles]

The policy MUST support `domain-core`, `application-shell`, `inbound-adapter`, `outbound-adapter`, `composition-root`, `test`, and `generated` roles.

#### Scenario: Production scope has one role per item

- GIVEN the policy declares its production crates, source scopes, and selected targets
- WHEN role resolution runs
- THEN every declared production crate and source scope MUST resolve to exactly one production role
- AND test and generated exclusions MUST remain explicit

#### Scenario: Role coverage is missing or ambiguous

- GIVEN a production path has no role or matches multiple role declarations
- WHEN role resolution runs
- THEN the policy result MUST contain a deterministic coverage or overlap diagnostic
- AND `gate` and `admission` modes MUST block

#### Scenario: Core host-independence rule is selected

- GIVEN a project requires `no_std` for one domain-core crate
- WHEN policy validation runs
- THEN Octet MUST record `no_std` as a separate host-independence obligation
- AND Octet MUST NOT treat `domain-core` as an implicit `no_std` requirement

### Requirement: Application owns port contracts

r[octet.project_architecture_policy.ports]

Declared inbound and outbound ports MUST name an application-owner scope and permitted adapter roles.

#### Scenario: Application-owned port is declared

- GIVEN a project declares a port trait in an `application-shell` scope and names permitted adapter scopes
- WHEN policy validation runs
- THEN the port declaration MUST be accepted for later compiler-backed evaluation

#### Scenario: Domain or adapter owns the port declaration

- GIVEN a declared port owner resolves to `domain-core`, `inbound-adapter`, `outbound-adapter`, or `composition-root`
- WHEN policy validation runs
- THEN strict project policy MUST reject the owner with a deterministic diagnostic

### Requirement: Architecture modes support staged adoption

r[octet.project_architecture_policy.modes]

Project policy MUST select exactly one mode from `inventory`, `advisory`, `gate`, or `admission`.

#### Scenario: Inventory and advisory report findings

- GIVEN a valid active policy selects `inventory` or `advisory`
- WHEN architecture evaluation finds a declared architecture violation
- THEN the result MUST retain the finding without reporting architecture-clean
- AND the architecture finding MUST NOT cause gate failure solely because of the selected mode

#### Scenario: Gate and admission block violations

- GIVEN a valid active policy selects `gate` or `admission`
- WHEN architecture evaluation finds a hard violation or required unknown fact
- THEN the architecture phase MUST block

#### Scenario: Required input is unavailable in any active mode

- GIVEN policy, export, Cargo, compiler, scope, or target input required by the selected policy is unavailable
- WHEN an active architecture run starts
- THEN the run MUST report an integration failure rather than an architecture-clean result

### Requirement: Architecture waivers are exact and visible

r[octet.project_architecture_policy.waivers]

An architecture waiver MUST bind a stable finding ID, policy identity, source BLAKE3, owner, reason, review condition, and affected claim.

#### Scenario: Exact current waiver applies

- GIVEN every waiver field matches one current architecture finding and source identity
- WHEN waiver evaluation runs
- THEN the finding MAY receive status `waived`
- AND the finding MUST remain visible in human and machine output

#### Scenario: Waiver is broad or stale

- GIVEN a waiver omits an exact binding, matches multiple findings, or references changed source or policy bytes
- WHEN waiver evaluation runs
- THEN the waiver MUST NOT suppress a finding
- AND strict modes MUST emit a blocking waiver diagnostic

### Requirement: Existing consumers remain compatible

r[octet.project_architecture_policy.compatibility]

Projects without an architecture pointer MUST retain existing Octet lint and strict-profile behavior.

#### Scenario: Deterministic core profile has no project policy

- GIVEN a project selects `--profile deterministic-core` and declares `core_source_scopes` but no architecture pointer
- WHEN `cargo octet check` runs
- THEN the direct strict-profile checks MUST run as before
- AND Octet MUST NOT infer or emit a full project architecture claim

#### Scenario: Octet checks its own layout

- GIVEN the Octet repository selects its explicit first-party project policy
- WHEN first-party architecture checks run
- THEN the selected obligations MUST preserve intended Octet self-check coverage without consumer fallbacks

### Requirement: Activation verification covers positive and negative paths

r[octet.project_architecture_policy.verification]

The implementation MUST include deterministic positive and negative fixtures for activation, schema, roles, modes, ports, waivers, and compatibility.

#### Scenario: Policy fixture matrix runs

- GIVEN valid, absent, malformed, ambiguous, uncovered, stale, and escaping policy fixtures
- WHEN focused policy validation runs
- THEN positive fixtures MUST produce their declared mode and role results
- AND negative fixtures MUST fail with their declared diagnostic class
