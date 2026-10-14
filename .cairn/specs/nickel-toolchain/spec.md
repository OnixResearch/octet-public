# Nickel Toolchain Specification

## Purpose

Defines the `nickel-toolchain` capability.

## Requirements

### Requirement: Octet pins Nickel 1.17 exactly

r[octet.nickel_toolchain.cohort] Octet MUST use Nickel `1.17.0` from upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426` for checked-path configuration.

#### Scenario: Checked-path tools resolve

- GIVEN Octet builds policy, standards, proof, and evidence checks
- WHEN cohort admission runs
- THEN every Nickel command MUST use the exact reviewed evaluator

#### Scenario: An older or floating evaluator resolves

- GIVEN an ambient package, branch, or older pin supplies Nickel
- WHEN admission runs
- THEN admission MUST fail

### Requirement: Checked-path authority remains Octet-owned

r[octet.nickel_toolchain.boundary] Nickel MUST validate configuration shape only. Octet MUST retain lint meaning, proof-rail policy, evidence roles, and findings.

#### Scenario: A profile passes

- GIVEN Nickel accepts an Octet profile
- WHEN a checked path runs
- THEN Octet MUST still apply its complete checker policy

### Requirement: Compatibility includes negative policy fixtures

r[octet.nickel_toolchain.compatibility] Tests MUST cover valid profiles and rejected disabled, duplicate, empty, warning-mode, malformed, import, contract, and unknown-value fixtures.

#### Scenario: The fixture matrix runs

- GIVEN representative valid and invalid profiles
- WHEN Nickel `1.17.0` evaluates them
- THEN valid profiles MUST retain supported outcomes
- AND invalid profiles MUST fail closed

### Requirement: Evidence keeps narrow claims

r[octet.nickel_toolchain.evidence] Evidence affected by Nickel MUST record the exact evaluator identity. It MUST NOT promote syntax conformance into semantic proof.

#### Scenario: Evidence is generated

- GIVEN a Nickel-backed rail passes
- WHEN evidence is emitted
- THEN it MUST bind the evaluator version and source commit
- AND existing verification scopes MUST remain explicit

### Requirement: The repository validation rail passes

r[octet.nickel_toolchain.validation] The change MUST pass policy fixtures, standards checks, proof-profile checks, formatting, Clippy, Cairn gates, and relevant Nix checks.

#### Scenario: Validation completes

- GIVEN the pin, fixtures, and evidence are current
- WHEN validation runs
- THEN every required check MUST pass or report one exact blocker
