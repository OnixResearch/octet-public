# Compiler Architecture Ir Specification

## Purpose

Defines the `compiler-architecture-ir` capability.

## Requirements

### Requirement: Compiler architecture collection follows project activation

r[octet.compiler_architecture_ir.activation]

Octet MUST collect compiler architecture facts only for a valid active project architecture policy.

#### Scenario: Active policy starts collection

- GIVEN a project selects a valid architecture policy and declared production targets
- WHEN `cargo octet check` invokes Cargo
- THEN the runner MUST activate compiler fact collection for the selected compilation units

#### Scenario: No policy leaves compilation unchanged

- GIVEN a project has no architecture-policy pointer
- WHEN `cargo octet check` invokes Cargo
- THEN Octet MUST NOT create architecture shard directories, collector environment, Architecture IR, or coverage artifacts

### Requirement: Architecture IR schemas are Rust-owned

r[octet.compiler_architecture_ir.schema]

Octet MUST define versioned Rust DTOs for compiler fact shards, normalized Architecture IR, collection coverage, diagnostics, and receipt metadata.

#### Scenario: Generated contract is current

- GIVEN the Rust DTO schema is current
- WHEN contract generation and freshness validation run
- THEN the generated Nickel contract MUST match the Rust-owned schema
- AND the receipt-family registry MUST classify the output as Rust-derived

#### Scenario: Shard schema is malformed

- GIVEN a shard has an unknown schema, missing identity, invalid reference, invalid bound, or malformed span
- WHEN pure normalization runs
- THEN normalization MUST reject the shard with a deterministic diagnostic

### Requirement: Compiler facts cover architecture semantics

r[octet.compiler_architecture_ir.facts]

Each selected compilation-unit shard MUST record bounded compiler facts for roles, items, types, traits, implementations, signatures, calls, constructors, effects, expansions, and active configuration.

#### Scenario: Resolved architecture facts are emitted

- GIVEN rustc resolves a public signature, trait implementation, constructor type, call definition, macro origin, or direct effect primitive
- WHEN the collector evaluates the unit
- THEN the shard MUST record the resolved definition origin, source span, active target, feature set, and fact kind
- AND workspace source paths MUST be normalized and workspace-relative
- AND external generated sources MUST use typed BLAKE3 provenance without a workspace path

#### Scenario: Source text mimics a declaration

- GIVEN a comment, string, inactive configuration branch, or unresolved source spelling contains an expected trait or implementation name
- WHEN the collector runs
- THEN that spelling MUST NOT satisfy a compiler-fact obligation

### Requirement: Architecture identities are canonical and content-bound

r[octet.compiler_architecture_ir.identity]

Durable Architecture IR identities MUST use domain-separated BLAKE3 framing over canonical project, policy, package, target, item, signature, and source facts.

#### Scenario: Unchanged inputs repeat

- GIVEN policy, source, Cargo graph, features, target, toolchain, and compiler facts are unchanged
- WHEN collection and normalization run twice
- THEN shard identities, normalized ordering, coverage, and final Architecture IR identity MUST match

#### Scenario: Compiler-local index changes

- GIVEN a rustc-local HIR, MIR, or definition index changes while canonical semantic and source facts remain equal
- WHEN durable identities are computed
- THEN the local index MUST NOT become part of the durable identity

### Requirement: Collection coverage is complete

r[octet.compiler_architecture_ir.coverage]

Octet MUST compare expected production compilation units with valid observed compiler shards before it emits complete Architecture IR.

#### Scenario: Every expected unit is observed

- GIVEN locked Cargo metadata, policy targets, exact invocation, and valid shards identify the same complete unit set
- WHEN coverage evaluation runs
- THEN coverage MUST report complete with every expected and observed unit identity

#### Scenario: Unit is missing or mismatched

- GIVEN an expected target emits no shard or a shard has a stale package, target, feature, source, policy, toolchain, or collector identity
- WHEN coverage evaluation runs
- THEN collection MUST fail with a deterministic incomplete-coverage diagnostic

### Requirement: Unsupported compiler facts remain explicit

r[octet.compiler_architecture_ir.unknowns]

The collector MUST emit typed unknown records for ambiguous dispatch, unsupported expansion, opaque generated code, missing bodies, FFI, and unsupported compiler shapes.

#### Scenario: Dynamic dispatch is unresolved

- GIVEN a selected call uses trait-object or other dynamic dispatch that the collector cannot resolve exactly
- WHEN collection runs
- THEN the caller MUST receive a typed unknown call fact with its source and target identity

#### Scenario: Unknown fact reaches normalization

- GIVEN a valid shard contains an unknown fact
- WHEN normalization runs
- THEN the unknown fact MUST remain in canonical output
- AND normalization MUST NOT infer a clean or effect-free fact

### Requirement: Compiler collection is bounded and scoped

r[octet.compiler_architecture_ir.boundary]

The project policy MUST define named limits for units, shards, items, types, traits, implementations, calls, effects, unknowns, spans, and artifact bytes.

#### Scenario: Collection stays within limits

- GIVEN every fact count and byte size is within policy limits
- WHEN collection completes
- THEN the result MAY emit canonical Architecture IR with explicit non-claims

#### Scenario: A limit is exceeded

- GIVEN a collector or normalizer exceeds one declared limit
- WHEN the limit is detected
- THEN collection MUST stop or truncate only as declared
- AND the result MUST be incomplete and MUST NOT support a gate-clean claim

### Requirement: Compiler fact decisions use a functional core

r[octet.compiler_architecture_ir.functional_core]

Shard validation, identity calculation, normalization, coverage, and diagnostic ordering MUST operate over explicit in-memory values.

#### Scenario: Normalizer runs without compiler effects

- GIVEN policy, expected-unit, and shard values are supplied directly
- WHEN normalizer tests run
- THEN they MUST not require filesystem access, environment values, Cargo, rustc processes, clocks, randomness, or output sinks

#### Scenario: Shell owns collection effects

- GIVEN the runner invokes Cargo, coordinates shards, reads files, or writes artifacts
- WHEN reviewers inspect the implementation
- THEN those effects MUST remain in the cargo-octet or rustc-adapter shell

### Requirement: Compiler IR verification is adversarial

r[octet.compiler_architecture_ir.verification]

The implementation MUST include positive and negative fixtures for compiler semantics, identity, unknowns, bounds, coverage, activation, and deterministic replay.

#### Scenario: Adversarial fixture matrix runs

- GIVEN fixtures include aliases, comments, macros, inactive code, generic signatures, trait implementations, indirect factories, dynamic dispatch, missing units, stale shards, and malformed facts
- WHEN focused collector validation runs
- THEN positive fixtures MUST emit expected canonical facts
- AND negative fixtures MUST fail with their declared diagnostic class
