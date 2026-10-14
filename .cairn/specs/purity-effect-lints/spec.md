# Purity Effect Lints Specification

## Purpose

Defines the `purity-effect-lints` capability.

## Requirements

### Requirement: Ambient environment lint

r[octet.purity_effect_lints.ambient_env] Octet MUST provide an `ambient_env` lint that flags direct production reads or mutations of process environment state and guides maintainers toward explicit input parameters or shell-boundary capture.

#### Scenario: Ambient environment access is flagged

r[octet.purity_effect_lints.ambient_env.flags_env]

- GIVEN production Rust code directly calls a known environment API such as env var reads, process argument reads, current-directory reads, or environment mutation
- WHEN the lint runs
- THEN `ambient_env` MUST emit a diagnostic at the environment access.
- AND the diagnostic MUST explain that environment state is hidden input or output.

#### Scenario: Explicit input and test setup stay clean

r[octet.purity_effect_lints.ambient_env.allows_explicit_input]

- GIVEN code receives configuration, paths, or arguments as explicit parameters, or the access occurs in a recognized test context or justified shell helper
- WHEN the lint runs
- THEN `ambient_env` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.ambient_env.catalog]

- GIVEN `ambient_env` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Ambient randomness lint

r[octet.purity_effect_lints.ambient_random] Octet MUST provide an `ambient_random` lint that flags direct ambient randomness or non-deterministic ID generation in production code and guides maintainers toward explicit seeds, injected generators, or shell-generated values.

#### Scenario: Ambient randomness is flagged

r[octet.purity_effect_lints.ambient_random.flags_random]

- GIVEN production Rust code directly calls a known thread-local RNG, OS randomness, `getrandom`, or UUID v4 generation API
- WHEN the lint runs
- THEN `ambient_random` MUST emit a diagnostic at the random source.
- AND the diagnostic MUST explain that randomness is hidden input for deterministic core logic.

#### Scenario: Injected randomness stays clean

r[octet.purity_effect_lints.ambient_random.allows_injected_randomness]

- GIVEN code receives an RNG trait object, seed value, nonce bytes, or generated identifier as an explicit input
- WHEN the lint runs
- THEN `ambient_random` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.ambient_random.catalog]

- GIVEN `ambient_random` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

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

### Requirement: Impure call in core lint

r[octet.purity_effect_lints.impure_call_in_core] Octet MUST provide an `impure_call_in_core` lint that flags direct platform-effect calls inside configured `core_source_scopes` and guides maintainers toward injected ports or shell-boundary helpers.

#### Scenario: Effectful platform API is flagged in core scope

r[octet.purity_effect_lints.impure_call_in_core.flags_effect]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it directly calls a known filesystem, IO, network, process, environment, random, async-runtime, or platform-effect API
- THEN `impure_call_in_core` MUST emit a diagnostic at the effectful call site.
- AND the diagnostic MUST name the effect family and suggest injecting a port/capability or moving the call to a shell boundary.

#### Scenario: Shell boundary and injected capability stay clean

r[octet.purity_effect_lints.impure_call_in_core.allows_boundary]

- GIVEN a Rust source file is outside configured core source scopes or only calls an injected trait/capability inside a core scope
- WHEN the lint runs
- THEN `impure_call_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.impure_call_in_core.catalog]

- GIVEN `impure_call_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

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

### Requirement: Logging in core lint

r[octet.purity_effect_lints.logging_in_core] Octet MUST provide a `logging_in_core` lint that flags direct printing, debugging, logging, tracing, or metrics emission inside configured core source scopes.

#### Scenario: Core observability side effect is flagged

r[octet.purity_effect_lints.logging_in_core.flags_logging]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it directly prints, debugs, logs, traces, or emits metrics through a known macro or call path
- THEN `logging_in_core` MUST emit a diagnostic at the user-written site.
- AND the diagnostic MUST recommend returning structured events or using a shell-owned observability port.

#### Scenario: Returned events and shell logging stay clean

r[octet.purity_effect_lints.logging_in_core.allows_events]

- GIVEN core code returns structured events or shell code records logs/metrics
- WHEN the lint runs
- THEN `logging_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.logging_in_core.catalog]

- GIVEN `logging_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Missing const fn lint

r[octet.purity_effect_lints.missing_const_fn] Octet SHOULD provide a `missing_const_fn` lint that identifies conservative, const-compatible helper functions that are not declared `const fn`.

#### Scenario: Const-eligible helper is flagged

r[octet.purity_effect_lints.missing_const_fn.flags_const_eligible]

- GIVEN a Rust helper function is not declared `const`
- WHEN its signature and body match Octet's conservative const-eligible subset
- THEN `missing_const_fn` SHOULD emit an advisory diagnostic at the function signature.
- AND any suggestion MUST use an applicability level that reflects whether compile-proof or autofix proof exists.

#### Scenario: Non-const or intentionally flexible code stays clean

r[octet.purity_effect_lints.missing_const_fn.allows_non_const]

- GIVEN a function is async, allocates, calls non-const functions, performs shell effects, implements a trait method, is already `const`, or falls under a public-API guardrail
- WHEN the lint runs
- THEN `missing_const_fn` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.missing_const_fn.catalog]

- GIVEN `missing_const_fn` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Mutating input in pure lint

r[octet.purity_effect_lints.mutating_input_in_pure] Octet MUST provide a `mutating_input_in_pure` lint that flags function parameters capable of mutating caller-owned state in declared pure contexts.

#### Scenario: Mutating input capability is flagged

r[octet.purity_effect_lints.mutating_input_in_pure.flags_mutating_input]

- GIVEN a function is in a configured pure/core context
- WHEN its signature accepts `&mut`, raw pointer parameters, or parameters typed as locks, atomics, cells, or unsafe cells
- THEN `mutating_input_in_pure` MUST emit a diagnostic at the hazardous parameter.
- AND the diagnostic MUST recommend an owned state-transition input/output or moving the mutation to an imperative shell.

#### Scenario: Explicit state transition stays clean

r[octet.purity_effect_lints.mutating_input_in_pure.allows_explicit_transition]

- GIVEN a function receives owned state, borrows immutable input, mutates only local temporaries, or lives in a shell scope
- WHEN the lint runs
- THEN `mutating_input_in_pure` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.mutating_input_in_pure.catalog]

- GIVEN `mutating_input_in_pure` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Nondeterministic output lint

r[octet.purity_effect_lints.nondeterministic_output] Octet SHOULD provide a `nondeterministic_output` lint that flags obvious unordered hash-collection iteration flowing into ordered outputs without a stable sort.

#### Scenario: Unsorted hash-derived ordered output is flagged

r[octet.purity_effect_lints.nondeterministic_output.flags_unsorted_hash_output]

- GIVEN Rust code iterates a `HashMap` or `HashSet`
- WHEN the iterated values are directly collected, joined, serialized, or rendered into an ordered output without a stable sort
- THEN `nondeterministic_output` SHOULD emit a diagnostic at the ordered-output construction.
- AND the diagnostic MUST recommend sorting by a stable key or using an ordered collection type.

#### Scenario: Stable ordering stays clean

r[octet.purity_effect_lints.nondeterministic_output.allows_stable_order]

- GIVEN code sorts hash-derived values before ordered output, uses `BTreeMap` or `BTreeSet`, or returns an explicitly unordered collection
- WHEN the lint runs
- THEN `nondeterministic_output` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.nondeterministic_output.catalog]

- GIVEN `nondeterministic_output` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Pure attribute contract lint

r[octet.purity_effect_lints.pure_attr_contract] Octet MUST provide a `pure_attr_contract` lint that enforces direct-effect purity checks for functions marked with Octet's recognized pure-function marker.

#### Scenario: Marked impure function is flagged

r[octet.purity_effect_lints.pure_attr_contract.flags_marked_impurity]

- GIVEN a Rust function is marked with the recognized pure-function marker
- WHEN its signature or body directly uses a banned effect such as platform IO, ambient time/env/random, logging, spawning, global state, interior mutability, mutating input capabilities, panic paths, or resource handles
- THEN `pure_attr_contract` MUST emit a diagnostic at the violating source shape.
- AND the diagnostic MUST explain that the marker enforces direct source-shape checks rather than transitive whole-program purity proof.

#### Scenario: Clean marked pure function stays clean

r[octet.purity_effect_lints.pure_attr_contract.allows_clean_marked_pure]

- GIVEN a marked function computes from explicit inputs, returns explicit outputs, and avoids the banned direct-effect checklist
- WHEN the lint runs
- THEN `pure_attr_contract` MUST NOT emit for that function.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.pure_attr_contract.catalog]

- GIVEN `pure_attr_contract` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, marker syntax docs, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Resource return from core lint

r[octet.purity_effect_lints.resource_return_from_core] Octet MUST provide a `resource_return_from_core` lint that flags functions in configured core source scopes whose return types contain concrete resource or capability handles.

#### Scenario: Core resource return is flagged

r[octet.purity_effect_lints.resource_return_from_core.flags_resource_return]

- GIVEN a function is declared inside a configured core source scope
- WHEN its return type contains a known file, socket, process, command, task, channel, connection, or external-client handle
- THEN `resource_return_from_core` MUST emit a diagnostic at the return type.
- AND the diagnostic MUST recommend returning domain values, command descriptions, or plans that a shell executes.

#### Scenario: Descriptions and shell resources stay clean

r[octet.purity_effect_lints.resource_return_from_core.allows_descriptions]

- GIVEN shell code returns resource handles or core code returns command-description enums, serializable plans, domain values, or port-independent data
- WHEN the lint runs
- THEN `resource_return_from_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.resource_return_from_core.catalog]

- GIVEN `resource_return_from_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Spawn in core lint

r[octet.purity_effect_lints.spawn_in_core] Octet MUST provide a `spawn_in_core` lint that flags direct thread or async task spawning inside configured core source scopes.

#### Scenario: Core spawn is flagged

r[octet.purity_effect_lints.spawn_in_core.flags_spawn]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it directly calls a known thread or async runtime spawn API
- THEN `spawn_in_core` MUST emit a diagnostic at the spawn call.
- AND the diagnostic MUST recommend returning a command/work description or using a shell-owned executor port.

#### Scenario: Shell executor ownership stays clean

r[octet.purity_effect_lints.spawn_in_core.allows_shell_executor]

- GIVEN shell code spawns work or core code returns command values or uses an injected executor abstraction without direct spawning
- WHEN the lint runs
- THEN `spawn_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.spawn_in_core.catalog]

- GIVEN `spawn_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.

### Requirement: Zero-alloc attribute contract lint

r[octet.purity_effect_lints.zero_alloc_attr_contract] Octet MUST provide a `zero_alloc_attr_contract` lint that flags direct heap-allocation sites in functions marked with Octet's recognized zero-allocation marker and guides maintainers toward stack-local construction, pre-allocated buffers, or shell-owned allocation.

#### Scenario: Marked allocating function is flagged

r[octet.purity_effect_lints.zero_alloc_attr_contract.flags_allocation]

- GIVEN a Rust function is marked with the recognized zero-allocation marker (`#[tigerstyle::zero_alloc]`, `#[octet::zero_alloc]`, or `#[zero_alloc]`)
- WHEN its body directly constructs a heap-allocating value such as `Vec::new()`, `Box::new(..)`, `String::from(..)`, `.to_string()`, `format!(..)`, or another allocator constructor covered by Octet's shared allocation classifier
- THEN `zero_alloc_attr_contract` MUST emit a diagnostic at the allocation site.
- AND the diagnostic MUST state that the marker enforces direct source-shape checks on allocation sites rather than transitive whole-call-tree non-allocation proof.

#### Scenario: Non-allocating or explicitly-allowed function stays clean

r[octet.purity_effect_lints.zero_alloc_attr_contract.allows_non_allocating]

- GIVEN a marked function computes from stack locals, array initialization, primitive returns, or struct construction of stack-only fields
- WHEN the lint runs
- THEN `zero_alloc_attr_contract` MUST NOT emit for that function.
- AND a function that legitimately allocates MAY silence the lint with an explicit `allow` paired with a justification.

#### Scenario: Auditable escape hatch rejects silent allow

r[octet.purity_effect_lints.zero_alloc_attr_contract.auditable_escape_hatch]

- GIVEN a function uses a bare `#[allow(tigerstyle::zero_alloc_attr_contract)]` or `#[allow(zero_alloc_attr_contract)]` without a justification comment
- WHEN the `unjustified_allow` lint runs
- THEN `unjustified_allow` MUST emit for that allow.
- AND Octet MUST NOT provide a silent `assume`-style attribute that disables `zero_alloc_attr_contract` without a visible diff artifact.

#### Scenario: Allocation classifier is shared with the loop lint

r[octet.purity_effect_lints.zero_alloc_attr_contract.shared_classifier]

- GIVEN `zero_alloc_attr_contract` and `allocation_in_loop` both classify allocator constructors
- WHEN the shared allocation classifier is updated to recognize a new allocator type
- THEN both lints MUST classify that type consistently and their UI fixtures MUST be updated together.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.zero_alloc_attr_contract.catalog]

- GIVEN `zero_alloc_attr_contract` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, marker syntax docs, and downstream rollout guidance MUST include the lint identity and default level.
