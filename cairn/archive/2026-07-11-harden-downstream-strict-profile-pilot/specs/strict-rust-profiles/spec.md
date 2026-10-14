# Strict Rust Profiles Specification

## Purpose

Hardens Octet strict-profile execution for dependency-bearing downstream functional cores.

## Requirements

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
