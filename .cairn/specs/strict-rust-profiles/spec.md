# Strict Rust Profiles Specification

## Purpose

Defines the `strict-rust-profiles` capability.

## Requirements

### Requirement: Nickel-authored strict-profile catalog

r[octet.strict_rust_profiles.catalog] Octet MUST provide a typed Nickel-authored strict-profile catalog with checked-in generated runtime JSON, a stable schema version, unique profile IDs, required lint identities, core-scope requirements, function-address floors, claim scope, and explicit non-claims.

#### Scenario: Valid catalog exports deterministically

r[octet.strict_rust_profiles.catalog.valid]

- GIVEN the reviewed strict-profile Nickel source declares supported profiles and fields
- WHEN Nickel typechecking and deterministic export run
- THEN the generated runtime JSON MUST match the checked-in artifact byte-for-byte.
- AND the catalog MUST contain unique `robust`, `deterministic-core`, and `bounded-systems` profile IDs.
- AND every required lint identity MUST exist in Octet's lint metadata catalog.

#### Scenario: Invalid catalog fails closed

r[octet.strict_rust_profiles.catalog.invalid]

- GIVEN a strict-profile fixture has an unsupported schema, empty profile ID, duplicate profile ID, unknown required lint, invalid function-address mode, empty non-claim set, or invalid core-scope combination
- WHEN Nickel or runtime catalog validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the invalid profile or field.

### Requirement: Explicit CLI profile selection

r[octet.strict_rust_profiles.selection] `cargo octet check` MUST accept `--profile <PROFILE>` and MUST resolve the profile from the embedded generated catalog before invoking Cargo.

#### Scenario: Known profile is selected

r[octet.strict_rust_profiles.selection.known]

- GIVEN an operator selects a shipped strict profile
- WHEN `cargo octet check --profile <PROFILE>` runs
- THEN the runner MUST pass the exact selected profile identity to the lint driver.
- AND the command MUST use that profile for enforcement, function-address policy, and run metadata.

#### Scenario: Unknown profile is rejected before Cargo

r[octet.strict_rust_profiles.selection.unknown]

- GIVEN an operator selects a profile ID absent from the generated catalog
- WHEN argument and profile validation run
- THEN the command MUST fail before Cargo execution.
- AND the diagnostic MUST name the unknown ID and list the supported profile IDs in deterministic order.

### Requirement: Profile lint floors

r[octet.strict_rust_profiles.lint_floor] A selected strict profile MUST apply its required lint identities at `deny` and MUST NOT permit consumer `disabled_lints` or weaker `lint_levels` values to reduce that profile floor.

#### Scenario: Required lints are strengthened

r[octet.strict_rust_profiles.lint_floor.strengthened]

- GIVEN consumer configuration omits, disables, allows, or warns a lint required by the selected profile
- WHEN the lint-driver configuration core applies the profile
- THEN the effective level for that lint MUST be `deny`.
- AND unrelated consumer lint levels and thresholds MUST remain unchanged.

#### Scenario: Source suppression remains explicit

r[octet.strict_rust_profiles.lint_floor.suppression]

- GIVEN source code uses an explicit lint suppression under a strict profile
- WHEN Octet checks the source
- THEN existing Octet suppression-justification lints MUST remain active according to the selected profile.
- AND the profile MUST NOT introduce a silent assume-style bypass.

### Requirement: Deterministic-core scope validation

r[octet.strict_rust_profiles.core_scope] The `deterministic-core` profile MUST require one or more configured `core_source_scopes` and MUST use those workspace-relative scopes as the default function-address scope.

#### Scenario: Configured core scopes are accepted

r[octet.strict_rust_profiles.core_scope.valid]

- GIVEN canonical consumer `dylint.toml` declares non-empty workspace-relative `core_source_scopes`
- WHEN `deterministic-core` profile preparation runs
- THEN scope resolution MUST preserve the declared order after deterministic de-duplication.
- AND those scopes MUST become the function-address paths when the operator supplied no explicit paths.

#### Scenario: Missing core scope fails before Cargo

r[octet.strict_rust_profiles.core_scope.missing]

- GIVEN `deterministic-core` is selected and consumer configuration has no core source scope
- WHEN profile preparation runs
- THEN the command MUST fail before Cargo execution with a diagnostic naming `core_source_scopes` and `deterministic-core`.

### Requirement: Function-address enforcement floor

r[octet.strict_rust_profiles.function_address_floor] Each strict profile MUST declare a minimum function-address mode, and profile preparation MUST preserve an operator-selected stronger mode while raising weaker modes to the profile floor.

#### Scenario: Profile raises weaker mode

r[octet.strict_rust_profiles.function_address_floor.raise]

- GIVEN a selected profile requires a stronger function-address mode than the command currently requests
- WHEN the effective check configuration is built
- THEN Octet MUST use the profile's minimum mode.
- AND `deterministic-core` MUST require deny-mode function-address evidence over its configured core paths.

#### Scenario: Stronger operator mode is preserved

r[octet.strict_rust_profiles.function_address_floor.preserve]

- GIVEN the operator explicitly selects a function-address mode stronger than the profile floor
- WHEN the effective check configuration is built
- THEN Octet MUST preserve the stronger mode and any explicit function-address paths or baseline.

### Requirement: Profile-bound deterministic evidence

r[octet.strict_rust_profiles.evidence] A strict-profile run MUST bind the selected profile ID and BLAKE3 identity into deterministic run metadata, summary output, provenance checked scope, finding configuration identity, and artifact verification.

#### Scenario: Selected profile is visible in evidence

r[octet.strict_rust_profiles.evidence.visible]

- GIVEN a strict-profile check produces an artifact bundle
- WHEN an operator inspects `summary.txt`, `status.json`, or `provenance.jsonl`
- THEN each surface MUST identify or cryptographically bind the selected profile.
- AND provenance checked scope MUST use `profile:<PROFILE>`.
- AND `cargo octet artifact verify` MUST accept an otherwise valid profile-bound bundle.

#### Scenario: Profile identity changes the run identity

r[octet.strict_rust_profiles.evidence.identity]

- GIVEN two checks use identical source, Cargo scope, and output format but select different strict profiles
- WHEN run metadata is computed
- THEN their profile and configuration identities MUST differ deterministically using BLAKE3.

### Requirement: Compatibility and claim boundaries

r[octet.strict_rust_profiles.boundaries] Strict-profile behavior MUST remain opt-in and MUST describe its result as bounded source-shape policy admission rather than a new Rust language, transitive effect system, complete dependency closure, formal proof, whole-program correctness, or release eligibility.

#### Scenario: Default check remains compatible

r[octet.strict_rust_profiles.boundaries.default]

- GIVEN an existing consumer invokes `cargo octet check` without `--profile`
- WHEN the runner prepares the check
- THEN existing `workspace-metadata` profile behavior, consumer configuration precedence, and function-address defaults MUST remain unchanged.

#### Scenario: Documentation preserves non-claims

r[octet.strict_rust_profiles.boundaries.non_claims]

- GIVEN an operator reads strict-profile help, documentation, or generated policy fields
- WHEN the supported claim is described
- THEN the text MUST state that the profile enforces configured direct source-shape checks and evidence binding only.
- AND it MUST NOT claim MIR call-graph closure, transitive runtime-effect absence, compiler correctness, verifier soundness, whole-program proof, dependency-closure certification, or release eligibility.

### Requirement: Positive and negative verification

r[octet.strict_rust_profiles.verification] The strict-profile implementation MUST include positive and negative tests for catalog validation, profile selection, lint-floor application, core-scope preparation, function-address floors, and evidence identity.

#### Scenario: Focused verification is reviewable

r[octet.strict_rust_profiles.verification.focused]

- GIVEN implementation is complete
- WHEN focused Rust tests, Nickel fixture checks, generated-artifact freshness, artifact verification, and Cairn validation run
- THEN positive cases MUST pass.
- AND negative cases MUST reject malformed or weakened inputs with deterministic diagnostics.
- AND the archived change MUST record the exact validation commands and outcomes.

### Requirement: Standalone downstream linker closure

r[octet.strict_rust_profiles.downstream_linker] The Nix-packaged `cargo-octet` runner MUST provide the linker tooling required to check ordinary downstream Rust crates with build scripts or proc-macro dependencies without inheriting an ambient consumer development shell.

#### Scenario: Proc-macro consumer reaches lint execution

r[octet.strict_rust_profiles.downstream_linker.proc_macro]

- GIVEN a downstream crate uses a proc-macro dependency and the pinned Octet Rust toolchain
- WHEN the packaged runner invokes Cargo outside the consumer dev shell
- THEN dependency compilation MUST find the packaged linker.
- AND the run MUST reach Octet lint and function-address processing rather than fail as an unclassified compiler diagnostic.

### Requirement: Core construction classification follows ownership

r[octet.strict_rust_profiles.core_construction_precision] `concrete_construction_in_core` MUST inspect direct constructor calls and MUST distinguish local or standard value construction from external concrete construction.

#### Scenario: External constructor is reported once

r[octet.strict_rust_profiles.core_construction_precision.external]

- GIVEN a declared core scope directly calls `ExternalType::new()` or `ExternalType::default()` for an external ADT
- WHEN the lint runs
- THEN Octet MUST emit exactly one diagnostic for that constructor call.

#### Scenario: Local and standard constructors pass

r[octet.strict_rust_profiles.core_construction_precision.local]

- GIVEN a declared core scope constructs a local domain DTO or a `core`, `alloc`, or `std` value
- WHEN the lint runs
- THEN Octet MUST NOT report `concrete_construction_in_core` for that constructor.

#### Scenario: Nested parents do not duplicate findings

r[octet.strict_rust_profiles.core_construction_precision.no_duplicates]

- GIVEN a direct external constructor appears inside a return, assignment, condition, or larger expression
- WHEN the lint walks parent expressions
- THEN only the direct constructor call MUST be eligible for a diagnostic.

### Requirement: Downstream hardening remains bounded

r[octet.strict_rust_profiles.downstream_hardening_boundaries] Downstream packaging and lint-precision evidence MUST remain bounded to tool availability and direct HIR/source-shape classification.

#### Scenario: Pilot evidence does not overclaim

r[octet.strict_rust_profiles.downstream_hardening_boundaries.non_claims]

- GIVEN the Valence downstream smoke passes
- WHEN Octet describes the result
- THEN it MUST NOT claim transitive effect absence, dependency closure, behavioral correctness, whole-program correctness, compiler correctness, verifier soundness, or release eligibility.
