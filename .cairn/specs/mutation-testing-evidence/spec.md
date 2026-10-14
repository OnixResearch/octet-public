# Mutation Testing Evidence Specification

## Purpose

Defines the `mutation-testing-evidence` capability.

## Requirements

### Requirement: Mutation runs use typed bounded profiles

r[octet.mutation_testing.profile]

Octet MUST run cargo-mutants only through a validated Nickel profile that binds provider identity, toolchain, Nix closure, source scope, features, exclusions, fixed argv, budgets, limits, retention policy, and non-claims.

#### Scenario: Reviewed profile constructs a run

- GIVEN a complete profile selects supported scope, provider, toolchain, settings, budgets, limits, and non-claims
- WHEN mutation run preflight executes
- THEN Octet MUST construct the declared argv deterministically
- AND the receipt MUST bind the profile, lockfile, toolchain, provider, source, and argv identities

#### Scenario: Arbitrary run authority fails

- GIVEN an operator supplies free-form command text, undeclared Cargo arguments, unsafe paths, ambient environment inputs, or a network-dependent provider
- WHEN preflight executes
- THEN Octet MUST reject the request before process execution

### Requirement: Mutation evidence uses native machine artifacts

r[octet.mutation_testing.native_artifacts]

Octet MUST normalize only the pinned provider's declared machine-readable artifact set and MUST retain raw artifact identities beside the normalization version.

#### Scenario: Supported native artifact set normalizes

- GIVEN a pinned cargo-mutants run produces its complete declared artifact set under policy limits
- WHEN the pure normalization core runs twice
- THEN both normalized inventories, outcomes, and diagnostics MUST match
- AND the receipt MUST bind every raw input with a BLAKE3 identity

#### Scenario: Only human logs exist

- GIVEN the provider emits only human stdout, stderr, terminal formatting, or an unsupported artifact version
- WHEN normalization runs
- THEN Octet MUST fail with an unsupported-provider or missing-artifact diagnostic
- AND it MUST NOT infer a passing result from human logs

### Requirement: Mutation evidence requires a clean baseline

r[octet.mutation_testing.baseline]

A profiled mutation receipt MUST bind a successful unmutated build and test baseline produced under the same source, lockfile, toolchain, target, feature, and profile scope.

#### Scenario: Matching baseline passes

- GIVEN the unmutated baseline completes successfully and every baseline identity matches the mutation run
- WHEN receipt classification runs
- THEN mutant outcomes MAY be evaluated under the selected policy

#### Scenario: Baseline is missing or unsuccessful

- GIVEN the baseline is missing, failed, timed out, stale, or scoped differently from the mutation run
- WHEN receipt classification runs
- THEN the receipt MUST fail or require manual review before mutant outcomes can support promotion

### Requirement: Mutation receipts preserve complete outcome partitions

r[octet.mutation_testing.receipt]

A mutation receipt MUST bind the candidate inventory and preserve caught, survived, timed-out, unviable, skipped, not-run, and tool-failure outcomes as distinct terminal classes.

#### Scenario: Complete outcome partition verifies

- GIVEN each generated mutant has exactly one supported terminal outcome
- WHEN receipt validation runs
- THEN validation MUST preserve mutant identity, operator, safe source location, source hash, outcome, diagnostics, and retained diff identity
- AND aggregate counts MUST equal the validated outcome partition

#### Scenario: Outcome partition is incomplete or contradictory

- GIVEN a mutant is missing, duplicated, assigned several terminal outcomes, references stale source, or conflicts with aggregate counts
- WHEN receipt validation runs
- THEN validation MUST fail with a deterministic inventory or outcome diagnostic

#### Scenario: Survivor remains visible

- GIVEN one or more mutants survive
- WHEN Octet renders the receipt
- THEN every survivor MUST remain independently reviewable
- AND an aggregate mutation score MUST NOT hide the survivor

### Requirement: Mutation evidence remains bounded test evidence

r[octet.mutation_testing.boundary]

Mutation receipts MUST remain optional `transcript_assertion` evidence and MUST NOT claim test completeness, source correctness, exhaustive mutation coverage, formal proof, or release eligibility.

#### Scenario: Legacy import remains bounded

- GIVEN an existing normalized mutation summary imports successfully without hardened run measurements
- WHEN Octet classifies it
- THEN the receipt MUST identify it as legacy imported transcript evidence
- AND Octet MUST NOT invent provider, baseline, inventory, or source measurements

#### Scenario: Starter checks run

- GIVEN no mutation profile is selected
- WHEN starter `cargo octet check` runs
- THEN cargo-mutants and its artifacts MUST NOT be required

### Requirement: Mutation hardening has positive and negative verification

r[octet.mutation_testing.verification]

The implementation MUST include deterministic positive and negative fixtures for profiles, baselines, native artifacts, inventories, outcomes, identities, limits, paths, policy decisions, compatibility, and non-claims.

#### Scenario: Focused mutation matrix completes

- GIVEN clean, caught, survived, timed-out, unviable, imported, failed-baseline, incomplete, duplicate, malformed, stale, unsafe, over-limit, and overclaim fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
