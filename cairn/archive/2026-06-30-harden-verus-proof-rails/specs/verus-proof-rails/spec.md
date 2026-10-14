# Verus Proof Rails Specification

## Purpose

Extends the `verus-proof-rails` capability with production-grade verifier identity, structured receipts, safer proof execution, deterministic matching, typed rail manifests, and operator feedback.

## Requirements

### Requirement: Production Verus verifier identity

r[octet.verus_proof_rails.verifier_identity] Octet MUST distinguish production Verus verifier identity from fixture-only verifier identity before emitting promotion-grade `formal_proof` evidence.

#### Scenario: Production verifier identity is complete

r[octet.verus_proof_rails.verifier_identity.production]

- GIVEN a Verus proof rail is eligible for production `formal_proof` provenance
- WHEN `cargo octet prove` records verifier identity for that rail
- THEN the identity MUST include the pinned verifier label, verifier binary digest, solver identity, Rust toolchain identity, and Nix closure or closure-graph digest.
- AND the identity MUST be recorded in the proof receipt and provenance record.

#### Scenario: Fixture verifier remains bounded

r[octet.verus_proof_rails.verifier_identity.fixture]

- GIVEN a proof run uses the fixture verifier wrapper
- WHEN Octet writes proof artifacts for that run
- THEN the receipt and summary MUST label the evidence as fixture-only.
- AND Octet MUST NOT promote that receipt as production `formal_proof` provenance unless the checked scope explicitly declares fixture-only proof plumbing.

### Requirement: Structured Verus proof receipts

r[octet.verus_proof_rails.structured_receipts] Octet MUST persist Verus proof receipts as structured Rust-owned records with deterministic validation before those receipts can support `formal_proof` provenance.

#### Scenario: Valid structured receipt passes

r[octet.verus_proof_rails.structured_receipts.valid]

- GIVEN a Verus proof rail has a structured receipt with schema version, policy ID, rail index, verifier identity, command identity, source bindings, proof bindings, trust boundaries, checked scope, pass signal, and proof artifact identity
- WHEN proof receipt validation runs
- THEN validation MUST accept the receipt.
- AND Octet MAY emit `formal_proof` provenance for the matching proof artifact.

#### Scenario: Malformed or stale structured receipt fails closed

r[octet.verus_proof_rails.structured_receipts.fail_closed]

- GIVEN a structured receipt omits a required field, uses an unsupported schema version, has stale source or proof bindings, has wrong verifier identity, omits proof artifact identity, or records fixture evidence as production evidence
- WHEN proof receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field.
- AND Octet MUST NOT emit `formal_proof` provenance for that receipt.

#### Scenario: Receipt contract is registered

r[octet.verus_proof_rails.structured_receipts.contract]

- GIVEN Octet emits structured Verus proof receipts from Rust code
- WHEN policy freshness validation runs
- THEN the Verus proof receipt contract MUST be registered as a Rust-derived receipt family.
- AND the committed Nickel contract MUST match the Rust-owned receipt schema source.

### Requirement: Safe Verus proof command execution

r[octet.verus_proof_rails.safe_execution] Octet MUST execute proof rails through a deterministic command model instead of an unchecked shell command string.

#### Scenario: Proof command uses argv and bounded environment

r[octet.verus_proof_rails.safe_execution.argv]

- GIVEN a selected Verus proof rail declares an executable and arguments
- WHEN `cargo octet prove` runs the rail
- THEN the CLI shell MUST execute the pinned verifier with explicit argv, a deterministic working directory, an allowlisted environment, a configured timeout, and bounded stdout/stderr artifact capture.
- AND the pure validation core MUST receive only typed command results and in-memory receipt inputs.

#### Scenario: Proof command failures fail closed

r[octet.verus_proof_rails.safe_execution.failures]

- GIVEN proof command spawning fails, exits unsuccessfully, times out, exceeds bounded log capture, or resolves to an unpinned verifier
- WHEN `cargo octet prove` reports the run
- THEN the proof row MUST fail with a deterministic diagnostic naming the command failure class.
- AND Octet MUST NOT emit `formal_proof` provenance for that failed run.

#### Scenario: Starter lint path remains independent

r[octet.verus_proof_rails.safe_execution.starter_independent]

- GIVEN a consumer runs ordinary `cargo octet check` under a starter profile
- WHEN no proof rails are selected
- THEN Octet MUST NOT require Verus, solver binaries, proof manifests, or proof artifacts.
- AND missing proof tooling MUST NOT affect the starter lint result.

### Requirement: Normalized proof rail path matching

r[octet.verus_proof_rails.path_matching] Octet MUST bind proof rails to normalized repo-relative source and proof paths using exact or glob semantics rather than substring matching.

#### Scenario: Exact and glob matches are deterministic

r[octet.verus_proof_rails.path_matching.deterministic]

- GIVEN a proof rail declares exact source paths, exact proof paths, or explicit glob patterns
- WHEN rail selection and receipt validation run
- THEN Octet MUST normalize candidate paths to repo-relative form before matching.
- AND Octet MUST bind each matched path deterministically in sorted order for receipt hashing and diagnostics.

#### Scenario: Ambiguous or overbroad matches fail closed

r[octet.verus_proof_rails.path_matching.ambiguous]

- GIVEN a rail path pattern is empty, matches no selected source, relies only on substring matching, or matches multiple paths where the rail requires a single artifact
- WHEN rail selection or verified-module coverage runs
- THEN Octet MUST report a deterministic diagnostic naming the path pattern.
- AND Octet MUST NOT treat the rail as proof coverage for the affected source.

### Requirement: Nickel proof rail manifest

r[octet.verus_proof_rails.rail_manifest] Human-authored Verus proof rail declarations SHOULD be maintained as typed Nickel and exported as deterministic JSON for runtime Rust consumers.

#### Scenario: Valid Nickel manifest exports deterministic rails

r[octet.verus_proof_rails.rail_manifest.valid]

- GIVEN a maintainer declares Verus proof rails in the Nickel manifest
- WHEN the manifest export command runs
- THEN the exported JSON MUST include backend, command argv, source paths, proof paths, verifier identity requirements, pass signal, trust boundaries, and receipt binding policy.
- AND repeated exports from the same manifest MUST produce byte-identical JSON.

#### Scenario: Invalid Nickel manifest fails before execution

r[octet.verus_proof_rails.rail_manifest.invalid]

- GIVEN a manifest omits required rail fields, declares an unsupported backend, uses empty source paths, or provides malformed trust boundaries
- WHEN manifest validation runs
- THEN validation MUST fail before `cargo octet prove` executes any verifier.
- AND the diagnostic MUST name the rejected manifest field.

### Requirement: Verus proof operator feedback

r[octet.verus_proof_rails.operator_feedback] Octet SHOULD provide list and explain surfaces for proof rails without broadening formal-proof claims.

#### Scenario: Selected rails can be listed

r[octet.verus_proof_rails.operator_feedback.list]

- GIVEN a workspace has exported Verus proof rails
- WHEN an operator runs the proof rail listing surface
- THEN Octet SHOULD show each selected rail's policy ID, backend, source scope, proof path, verifier label, and fixture-or-production mode.
- AND the output MUST include scoped non-claim wording for formal proof evidence.

#### Scenario: Unbacked claims can be explained

r[octet.verus_proof_rails.operator_feedback.explain]

- GIVEN a verified module or formal-proof claim lacks a matching validated receipt
- WHEN Octet reports or explains the claim
- THEN the output SHOULD identify the unmatched source, missing or stale rail field, required command, and remediation.
- AND Octet MUST NOT emit `formal_proof` provenance for the unbacked claim.
