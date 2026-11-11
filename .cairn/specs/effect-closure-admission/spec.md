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

r[octet.effect_closure.cli] Effect closure MUST run only after its required direct strict check and any activated project architecture gate succeed.

#### Scenario: Direct check, architecture gate, and closure are admitted

r[octet.effect_closure.cli.admitted]

- GIVEN the strict check passes, active project architecture gate passes, function addresses match, calls resolve, role edges are allowed, and closure effects satisfy policy
- WHEN `admission` mode or the explicit closure command runs
- THEN Octet MUST emit an admitted effect-closure bundle and identified parent evidence

#### Scenario: Direct or architecture failure blocks closure

r[octet.effect_closure.cli.check_failure]

- GIVEN the strict check fails or an activated project architecture gate blocks
- WHEN closure orchestration runs
- THEN Octet MUST NOT execute or claim an admitted closure
- AND status evidence MUST identify the failed prerequisite phase

#### Scenario: Explicit closure command has no project policy

- GIVEN a project selects the existing explicit closure command without a project architecture-policy pointer
- WHEN the command runs
- THEN existing strict-profile and closure behavior MUST remain compatible
- AND no project architecture claim MUST be inferred

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

- GIVEN selected source contains any macro invocation (including an attribute on the function, its parameters, its body, or an enclosing impl, trait, module, or file, other than an inert built-in attribute such as `doc`, `inline`, or a lint level), conditional compilation, async/await, ambiguous trait or method dispatch, unresolved call, unsupported syntax, or another shape outside the v1 resolver
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

r[octet.effect_closure.dependency_summaries] Every selected non-macro call not resolved to a selected local function by a call path that proves the target (a single-segment call to a free function in the caller's file and module, `Owner::name` to an inherent method of `Owner` in that scope, `self.name()` from an inherent method to the same inherent owner in that scope, or a path through `crate::`, `super::`, `self::`, module declarations, or local `use` bindings that reaches such a function as its only target, only through selected modules whose crate position is proven) MUST require one exact call-site summary bound to its caller function address, caller source path and module, every declaration the call path resolves through outside the caller (the `use` bindings of the blocks lexically enclosing the call, the enclosing header, and module, re-export, glob, and macro declarations reached; for a method call, every declaration that may bring its trait into scope), and resolved provider identity. Resolution MUST be bounded and deduplicated: each module and path is looked up once per referenced path, so glob cycles add nothing, and a path exceeding the lookup bound MUST be treated as unprovable. A call whose path goes through a module the selected sources cannot prove, through a name only a macro or attributed item could generate after a module path, or that sits in a module item declared in the function body MUST remain `unknown` and MUST NOT be summarized. V1 MUST NOT infer effect-free toolchain intrinsics from names such as `Ok`, `Some`, or `Vec`; macro calls remain directly `unknown` and cannot be summarized.

#### Scenario: Exact summary resolves an external edge

r[octet.effect_closure.dependency_summaries.valid]

- GIVEN a workspace policy names the stable call-site ID, review label, callee label, provider kind, exact package or toolchain version, target, sorted feature set, effect summary, and evidence reference
- WHEN the shell-resolved Cargo/toolchain graph exactly matches that provider identity
- THEN the call MAY contribute the declared effects and a reviewed-summary external edge to the closure receipt.
- AND the receipt MUST distinguish that summary from verified dependency implementation evidence.

#### Scenario: Unknown or stale dependency edge is denied

r[octet.effect_closure.dependency_summaries.invalid]

- GIVEN a call summary is missing, duplicated, unused, broad, declares `unknown`, lacks evidence, or mismatches package name, version, features, target, toolchain, workspace identity, caller function address, caller source path or module, or a declaration its call path resolves through (including a re-export in another selected module and, for a method call, the module's imports and traits)
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
- AND a stored receipt whose `schema_version` is not the current receipt schema (`octet-effect-closure-receipt/v2`) MUST be reported as an unsupported receipt schema before its body is decoded.

### Requirement: Compatibility and claim boundaries

r[octet.effect_closure.boundaries] Closure admission MUST remain opt-in and retain its conservative selected-source-token claim boundary when composed with project architecture evaluation.

#### Scenario: Existing check remains unchanged

r[octet.effect_closure.boundaries.default]

- GIVEN an operator invokes `cargo octet check` without project architecture activation
- WHEN the integrated architecture feature is installed
- THEN existing check arguments, lint behavior, address behavior, closure behavior, and artifact schemas MUST remain compatible

#### Scenario: Evidence does not overclaim

r[octet.effect_closure.boundaries.non_claims]

- GIVEN a parent architecture receipt links an effect-closure receipt
- WHEN output describes admission
- THEN each receipt MUST preserve its checker, inputs, evidence tier, assumptions, and non-claims
- AND their combination MUST NOT claim complete MIR closure, runtime determinism, whole-program correctness, or release eligibility

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

### Requirement: Kamacite semantic-emission profiles are explicit

r[octet.effect_closure.kamacite_mapping]

Octet MUST define an opt-in Nickel profile for one effect-closure profile and one canonical Kamacite closed effect summary.

Each semantic-emission rule MUST bind one exact function address or call-site, provider identity, Kamacite operation identity, evidence reference, and non-claims.

Ambient effect dispositions MUST remain a separate policy section.

#### Scenario: Exact semantic emission is declared

- GIVEN a reviewed emission site has one exact Kamacite operation identity and the closed row contains that identity
- WHEN semantic-emission validation runs
- THEN the rule MUST pass with the `semantic-emission` class.

#### Scenario: Semantic emission is ambiguous

- GIVEN one emission site matches multiple rules or one rule uses a broad unresolved scope
- WHEN semantic-emission validation runs
- THEN validation MUST fail with a deterministic ambiguous-semantic-emission diagnostic.

### Requirement: Ambient effects cannot become semantic emissions

r[octet.effect_closure.kamacite_dispositions]

Existing strict-profile ambient effect results MUST remain authoritative for Octet admission.

The integration MUST NOT discharge, rename, or convert a denied ambient effect into a Kamacite semantic operation.

The V1 profile MAY classify only `alloc` as `profile-local`. All other ambient effect classes MUST remain denied under `deterministic-core`.

#### Scenario: Allocation remains profile-local

- GIVEN the selected strict profile permits allocation and the reviewed profile classifies it as profile-local
- WHEN Octet compares closure and semantic-emission evidence with the Kamacite row
- THEN allocation MUST remain visible without becoming a semantic operation or disappearing from evidence.

#### Scenario: Ambient clock cannot become a semantic emission

- GIVEN selected source reads an ambient clock and a mapping rule names a semantic time operation
- WHEN direct closure and semantic-emission admission run
- THEN direct closure MUST remain denied and the mapping MUST NOT discharge or rename the clock effect.

#### Scenario: Unknown effect fails

- GIVEN closure evidence contains `unknown` or an unsupported ambient disposition
- WHEN comparison runs
- THEN admission MUST fail with a deterministic unmapped-ambient-effect diagnostic.

### Requirement: Comparison preserves ambient, emission, and declaration authority

r[octet.effect_closure.kamacite_comparison]

Octet MUST compare verified closure and semantic-emission evidence with the supplied Kamacite projection without rewriting any parent artifact.

The comparison MUST classify rows as `emitted-and-declared`, `profile-local`, `declared-not-emitted`, `emitted-not-declared`, `stale-identity`, `ambiguous-emission`, or `unknown`.

Only `emitted-and-declared`, `profile-local`, and `declared-not-emitted` MAY appear in an admitted result.

#### Scenario: Emitted undeclared operation fails

- GIVEN an exact semantic-emission site names an operation absent from the supplied closed effect row
- WHEN comparison runs
- THEN admission MUST fail with an `emitted-not-declared` result.

#### Scenario: Declared operation is not emitted

- GIVEN the Kamacite row contains an operation that selected Octet emission evidence does not include
- WHEN comparison runs
- THEN the result MUST record `declared-not-emitted` without claiming dead code, unreachable behavior, or complete coverage.

### Requirement: Kamacite comparison is an optional admission extension

r[octet.effect_closure.kamacite_cli]

`cargo octet admit --profile deterministic-core --closure` MAY accept `--kamacite-effect-profile PATH` after direct checks and effect closure pass.

The option MUST consume a projection bound to canonical Kamacite Preserves identities. It MUST NOT invoke Kamacite or change default admission behavior.

#### Scenario: Optional profile emits a handoff

- GIVEN direct checks, closure, semantic-emission admission, identity checks, and comparison all pass
- WHEN the operator supplies the Kamacite effect profile
- THEN Octet MUST emit the comparison and handoff artifacts with successful status.

#### Scenario: Default admission remains unchanged

- GIVEN the operator does not supply a Kamacite effect profile
- WHEN Octet runs existing check, admission, and artifact verification commands
- THEN arguments, behavior, receipts, exit status, and claim boundaries MUST remain compatible.

### Requirement: Kamacite handoff is deterministic and non-authoritative

r[octet.effect_closure.kamacite_handoff]

A successful comparison MUST emit a BLAKE3-bound handoff projection with exact closure, emission, profile, effect-summary, and operation identities.

The handoff MUST also bind function-address or provider scope, comparison class, diagnostics, producer role, and non-claims.

#### Scenario: Verified handoff is stable

- GIVEN identical closure, emission, profile, effect summary, and producer inputs
- WHEN Octet emits the handoff twice
- THEN canonical rows and the handoff identity MUST match.

#### Scenario: Parent identity is tampered

- GIVEN a handoff changes a closure, emission, profile, summary, operation, or producer identity after emission
- WHEN artifact verification runs
- THEN verification MUST fail with a deterministic parent-identity diagnostic.

### Requirement: Kamacite comparison claims remain bounded

r[octet.effect_closure.kamacite_boundary]

A passing comparison MUST prove only bounded agreement between selected Octet evidence and supplied Kamacite declaration artifacts.

It MUST NOT prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility.

#### Scenario: Semantic determinism claim fails

- GIVEN a handoff, receipt, report, help text, or document claims that comparison proves semantic or runtime determinism
- WHEN boundary validation runs
- THEN validation MUST fail with a deterministic effect-comparison-overclaim diagnostic.

### Requirement: Kamacite comparison has positive and negative fixtures

r[octet.effect_closure.kamacite_fixtures]

Octet MUST include positive, negative, property, compatibility, and frozen Kamacite fixtures for the emission and comparison path.

#### Scenario: Frozen Kamacite cohort drifts

- GIVEN a Kamacite schema, profile, effect-summary, operation identity, or non-claim field changes
- WHEN Octet validates the frozen cohort
- THEN validation MUST fail until an explicit reviewed cohort update binds the new identities.
