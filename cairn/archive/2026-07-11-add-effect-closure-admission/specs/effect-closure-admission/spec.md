# Effect Closure Admission Specification

## Purpose

Defines Octet's bounded, fail-closed transitive effect-closure admission for selected strict Rust core source.

## Requirements

### Requirement: Nickel-authored effect-closure policy

r[octet.effect_closure.catalog] Octet MUST provide a typed Nickel-authored catalog for effect vocabulary, strict-profile allowances, module roles, role edges, workspace call-summary contracts, receipt non-claims, and stable schema versions, with checked-in generated runtime input.

#### Scenario: Valid catalog exports deterministically

r[octet.effect_closure.catalog.valid]

- GIVEN the reviewed Nickel catalog and a conforming workspace policy
- WHEN Nickel validation and deterministic export run
- THEN the generated catalog JSON MUST match the checked-in runtime artifact byte-for-byte.
- AND the effect vocabulary MUST contain exactly `alloc`, `panic`, `unsafe`, `clock`, `random`, `fs`, `net`, `spawn`, `lock`, `ffi`, and `unknown` in canonical order.
- AND `deterministic-core` MUST allow `alloc` while denying every other effect in that vocabulary.

#### Scenario: Invalid policy fails closed

r[octet.effect_closure.catalog.invalid]

- GIVEN a catalog or workspace fixture has an unsupported schema, duplicate identity, unknown effect or role, missing non-claim, broad call matcher, unknown effect summary, non-canonical duplicate entry, or incomplete provider identity
- WHEN Nickel or runtime policy validation runs
- THEN validation MUST fail closed with deterministic diagnostics naming the invalid field and item.

### Requirement: Two-stage closure CLI

r[octet.effect_closure.cli] `cargo octet admit --profile deterministic-core --closure` MUST run the existing strict-profile check before attempting effect closure and MUST require successful deny-mode function-address evidence for the selected core paths.

#### Scenario: Direct check and closure are admitted

r[octet.effect_closure.cli.admitted]

- GIVEN the selected strict check passes, function addresses match, all source constructs and calls are resolved, role edges are allowed, and closure effects satisfy the profile
- WHEN the admission command runs
- THEN it MUST emit an admitted effect-closure bundle and exit successfully.
- AND the existing check artifacts MUST remain available as identified parent evidence.

#### Scenario: Direct check failure blocks closure

r[octet.effect_closure.cli.check_failure]

- GIVEN the strict check reports an error, stale function address, compiler failure, or integration failure
- WHEN the admission command runs
- THEN Octet MUST NOT claim or emit an admitted closure.
- AND status evidence MUST identify the failed direct-check phase.

### Requirement: Conservative direct-effect inference

r[octet.effect_closure.inference] Octet MUST deterministically infer direct source-token effect occurrences and syntactic call sites for every supported function in the selected core source envelopes.

#### Scenario: Direct effects are classified

r[octet.effect_closure.inference.direct]

- GIVEN selected Rust source contains allocation, panic-capable shapes, unsafe blocks, ambient clock or random access, filesystem or network access, spawning, locking, or foreign calls
- WHEN source-token inference runs
- THEN each covered occurrence MUST be assigned its canonical effect family and caller function address.
- AND hidden panic shapes covered by the policy, including indexing, panic macros, `unwrap`/`expect`, and panic-capable arithmetic, MUST contribute `panic`.

#### Scenario: Opaque resolution fails closed

r[octet.effect_closure.inference.unknown]

- GIVEN selected source contains any macro invocation, conditional compilation, async/await, ambiguous trait or method dispatch, unresolved call, unsupported syntax, or another shape outside the v1 resolver
- WHEN inference runs
- THEN the caller MUST acquire `unknown` and admission MUST be denied.
- AND an exact call-site summary MAY resolve only a call edge; it MUST NOT erase direct `unknown`, `unsafe`, `ffi`, `panic`, or another directly inferred effect.

### Requirement: Explicit module roles and edges

r[octet.effect_closure.roles] Every selected source path MUST have exactly one explicit workspace-policy role from `core`, `shell`, `adapter`, `ffi`, or `proof`, and every selected local call MUST follow the catalog's allowed role-edge matrix.

#### Scenario: Deterministic core calls admitted core

r[octet.effect_closure.roles.valid]

- GIVEN both caller and selected callee are explicitly assigned `core`
- WHEN a syntactic local edge resolves between them
- THEN the edge MUST be eligible for closure propagation.

#### Scenario: Missing or forbidden role edge is denied

r[octet.effect_closure.roles.invalid]

- GIVEN a selected path has no role, has multiple role declarations, uses an unknown role, or a local edge violates the catalog matrix
- WHEN admission runs
- THEN Octet MUST deny admission with deterministic path and edge diagnostics.

### Requirement: Exact dependency-call summaries

r[octet.effect_closure.dependency_summaries] Every selected non-macro call not unambiguously resolved to a selected local function MUST require one exact call-site summary bound to its caller function address and resolved provider identity. V1 MUST NOT infer effect-free toolchain intrinsics from names such as `Ok`, `Some`, or `Vec`; macro calls remain directly `unknown` and cannot be summarized.

#### Scenario: Exact summary resolves an external edge

r[octet.effect_closure.dependency_summaries.valid]

- GIVEN a workspace policy names the stable call-site ID, review label, callee label, provider kind, exact package or toolchain version, target, sorted feature set, effect summary, and evidence reference
- WHEN the shell-resolved Cargo/toolchain graph exactly matches that provider identity
- THEN the call MAY contribute the declared effects and a reviewed-summary external edge to the closure receipt.
- AND the receipt MUST distinguish that summary from verified dependency implementation evidence.

#### Scenario: Unknown or stale dependency edge is denied

r[octet.effect_closure.dependency_summaries.invalid]

- GIVEN a call summary is missing, duplicated, unused, broad, declares `unknown`, lacks evidence, or mismatches package name, version, features, target, toolchain, workspace identity, or caller function address
- WHEN admission runs
- THEN Octet MUST deny admission with an `unknown` or unadmitted/stale dependency-edge diagnostic.

#### Scenario: Ambient target selection is not evidence

r[octet.effect_closure.dependency_summaries.target]

- GIVEN the direct Cargo invocation does not contain an explicit `--target TARGET` argument
- WHEN closure attempts to resolve its provider graph
- THEN Octet MUST fail before effect closure rather than infer the host or trust ambient Cargo target configuration.
- AND an explicit target MUST bind both the direct Cargo invocation and the recorded provider graph.

### Requirement: Deterministic transitive fixed point

r[octet.effect_closure.transitive] Octet MUST compute each selected function's closure effects by monotonically propagating direct effects across selected local call edges until a deterministic finite-lattice fixed point is reached.

#### Scenario: Callee effect reaches every selected caller

r[octet.effect_closure.transitive.propagates]

- GIVEN selected function A calls selected function B and B has a direct or inherited effect
- WHEN closure is computed
- THEN A's closure effect set MUST include B's effect.
- AND nodes, edges, and effects MUST be emitted in canonical deterministic order.

#### Scenario: Cyclic graph converges

r[octet.effect_closure.transitive.cycle]

- GIVEN selected local functions form a recursive or mutually recursive cycle
- WHEN closure is computed
- THEN the finite effect sets MUST converge without depending on traversal order.
- AND the receipt MUST preserve the cycle without claiming termination or runtime reachability.

### Requirement: Profile-bound closure evidence

r[octet.effect_closure.evidence] A closure run MUST emit a versioned BLAKE3-addressed receipt binding the selected profile/config identities, function-address parent, selected source identities, workspace policy, Cargo lock/graph, provider identities, target/toolchain, role graph, direct effects, closure effects, diagnostics, and explicit non-claims.

#### Scenario: Admission evidence is visible and repeatable

r[octet.effect_closure.evidence.visible]

- GIVEN unchanged source, policy, function-address baseline, Cargo graph, profile catalog, target, and toolchain
- WHEN admission runs twice
- THEN the closure receipt identity and canonical node/edge/effect sets MUST match.
- AND `cargo octet admit verify --artifact-dir PATH` MUST report the stored bundle valid without rerunning Cargo.

#### Scenario: Tampering is rejected

r[octet.effect_closure.evidence.tamper]

- GIVEN a receipt, parent check artifact, source identity, workspace policy, dependency identity, status, summary, or provenance artifact is modified after admission
- WHEN admission verification runs
- THEN it MUST fail closed with a deterministic artifact or identity diagnostic.

### Requirement: Compatibility and claim boundaries

r[octet.effect_closure.boundaries] Closure admission MUST remain opt-in and MUST describe its result as conservative selected-source-token effect closure under exact reviewed dependency summaries.

#### Scenario: Existing check remains unchanged

r[octet.effect_closure.boundaries.default]

- GIVEN an operator invokes `cargo octet check` with or without an existing strict profile
- WHEN this change is installed
- THEN its arguments, default behavior, lint execution, address behavior, and artifact schema MUST remain compatible.

#### Scenario: Evidence does not overclaim

r[octet.effect_closure.boundaries.non_claims]

- GIVEN an operator reads help, documentation, status, summary, provenance, policy, or receipt output
- WHEN closure admission is described
- THEN Octet MUST state that v1 uses conservative selected source-token inference and reviewed summaries.
- AND it MUST NOT claim complete Rust name resolution, macro or trait-dispatch completeness, MIR call-graph completeness, dependency implementation verification, runtime reachability or ordering, panic freedom, semantic determinism, compiler correctness, whole-program correctness, verifier soundness, or release eligibility.

### Requirement: Valence deterministic-core dogfood

r[octet.effect_closure.dogfood] Octet MUST dogfood closure admission against the two production source files selected by Valence's shared-identity pilot without broad source suppressions.

#### Scenario: Initial probe fails closed visibly

r[octet.effect_closure.dogfood.initial]

- GIVEN Valence has no reviewed effect-closure call summaries
- WHEN the initial closure probe runs
- THEN unresolved or unadmitted call edges MUST deny admission.
- AND retained evidence MUST make the missing boundaries reviewable by stable call-site identity.

#### Scenario: Exact reviewed policy admits the bounded pilot

r[octet.effect_closure.dogfood.final]

- GIVEN Valence assigns both selected files the `core` role and provides exact identity-matching summaries for every non-local call
- WHEN strict check, closure admission, repeat replay, and artifact verification run
- THEN the bounded pilot MUST complete without closure errors.
- AND documentation MUST preserve that broader `valence-core` and dependency implementations remain unadmitted.

### Requirement: Positive and adversarial verification

r[octet.effect_closure.verification] The implementation MUST include positive and negative tests for policy validation, inference, call resolution, roles, dependency identity, fixed-point closure, CLI orchestration, evidence identity, compatibility, and verification.

#### Scenario: Adversarial fixture matrix is reviewable

r[octet.effect_closure.verification.adversarial]

- GIVEN fixtures cover local calls, external dependencies, macros, traits and dynamic dispatch, `cfg`, FFI, unsafe, hidden panic paths, recursive cycles, stale function addresses, stale or extra summaries, target/feature/version mismatch, and artifact tampering
- WHEN focused tests and saved dogfood replay run
- THEN positive cases MUST pass deterministically.
- AND negative cases MUST deny admission with the expected diagnostic class rather than panic, silently drop an edge, or weaken the profile.
