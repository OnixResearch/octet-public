# Nickel Export Helpers Specification

## Purpose

Defines the `nickel-export-helpers` capability.

## Requirements

### Requirement: Deterministic Nickel export helper

r[octet.nickel_export_helpers.export] Octet SHOULD provide a deterministic helper for exporting Nickel-authored configuration or policy artifacts into the native file format consumed by the next tool or gate.

#### Scenario: Exported bytes are stable

r[octet.nickel_export_helpers.export.stable]

- GIVEN a Nickel source file, its declared import dependencies, an optional field selector, an optional contract, an output format, and an output path
- WHEN the Nickel export helper runs twice with the same inputs and Nickel binary identity
- THEN the exported artifact bytes SHOULD be identical.
- AND the helper SHOULD support only declared formats such as JSON, TOML, YAML, or raw text.

#### Scenario: Contract failure blocks export acceptance

r[octet.nickel_export_helpers.export.contract_failure]

- GIVEN an export declares a Nickel contract for the authored configuration
- WHEN the source value fails that contract
- THEN the export MUST fail before the generated artifact is accepted as fresh.
- AND the diagnostic SHOULD name the failing source or contract field when Nickel reports it.

### Requirement: Nickel export manifest freshness

r[octet.nickel_export_helpers.manifest] Helper-produced exports MUST be covered by a deterministic BLAKE3-backed manifest that can reject stale or mixed-generation artifacts.

#### Scenario: Manifest records source and output identity

r[octet.nickel_export_helpers.manifest.records]

- GIVEN the Nickel export helper writes an artifact
- WHEN it writes or updates the export manifest
- THEN the manifest MUST record source paths, dependency paths, output format, output path, byte length, BLAKE3 output digest, generation command, and source or dependency digests.
- AND the manifest SHOULD record Nickel binary identity when available.

#### Scenario: Stale export fails verification

r[octet.nickel_export_helpers.manifest.stale]

- GIVEN a Nickel source, dependency, or generated artifact changes without refreshing the export manifest
- WHEN export freshness verification runs
- THEN verification MUST fail with a diagnostic naming the stale source, dependency, or generated artifact.

### Requirement: Nickel export boundaries

r[octet.nickel_export_helpers.boundaries] Nickel export helpers MUST preserve Octet's generation-time boundary and MUST NOT make ordinary runtime checks depend on dynamic Nickel evaluation or secret-bearing generated artifacts.

#### Scenario: Runtime runner reads generated artifacts

r[octet.nickel_export_helpers.boundaries.runtime]

- GIVEN a consumer runs ordinary `cargo octet check` with generated policy artifacts already present
- WHEN the runner loads configuration or policy inputs
- THEN the runner SHOULD consume deterministic generated artifacts rather than evaluating Nickel dynamically.
- AND a Nickel binary SHOULD only be required for explicit generation or freshness-validation commands.

#### Scenario: Secret-bearing export is excluded

r[octet.nickel_export_helpers.boundaries.secrets]

- GIVEN a Nickel-authored configuration contains raw secrets, credentials, private keys, or other secret material
- WHEN an export family is considered for manifest-backed generated artifacts
- THEN the family MUST be rejected or registered as a non-candidate with an explicit exclusion rationale.

### Requirement: Cross-stack export inventory precedes extraction

r[octet.nickel_export_helpers.repository_extraction.inventory] Octet MUST record a reviewed inventory of Nickel export behavior in Octet, Mantle, Cairn, Trellis, and Animus before selecting the standalone contract.

#### Scenario: Inventory separates shared and retained behavior

r[octet.nickel_export_helpers.repository_extraction.inventory.boundary]
- GIVEN the stack implementations use external and embedded evaluators with different product policies
- WHEN the extraction inventory is reviewed
- THEN it MUST name shared request, identity, receipt, freshness, format, and diagnostic behavior
- AND it MUST identify evaluator execution, product semantic validation, destination writes, runtime policy, and release gates that remain consumer-owned.

### Requirement: Standalone repository owns the canonical contract

r[octet.nickel_export_helpers.repository_extraction.repository] Octet MUST move canonical evaluator-neutral Nickel export contract ownership to an independent `OnixResearch/nickel-export` repository with its own lifecycle, release profile, documentation, and source identity.

#### Scenario: Standalone package has no product dependency

r[octet.nickel_export_helpers.repository_extraction.repository.independent]
- GIVEN the standalone repository is prepared for release
- WHEN its package graph and repository configuration are validated
- THEN it MUST NOT depend on Octet lint execution, Mantle build/store internals, Cairn lifecycle internals, Trellis proofs, Animus runtime code, or workspace-relative sibling paths
- AND its release metadata MUST identify the canonical source and verification entrypoints.

### Requirement: Export decisions use a functional core

r[octet.nickel_export_helpers.repository_extraction.functional_core] The standalone repository MUST provide a deterministic functional core over explicit in-memory export inputs and MUST keep filesystem, process, environment, clock, network, and output effects outside that core.

#### Scenario: Embedded evaluator uses the core without a process shell

r[octet.nickel_export_helpers.repository_extraction.functional_core.embedded]
- GIVEN Mantle or another consumer has already evaluated Nickel and supplies exact source, dependency, output, evaluator, and diagnostic facts
- WHEN it calls the standalone core
- THEN manifest, receipt, acceptance, and freshness results MUST be computed without spawning `nickel` or reading ambient host state.

#### Scenario: Ambient authority in the core is rejected

r[octet.nickel_export_helpers.repository_extraction.functional_core.purity]
- GIVEN a core change introduces file reads, process execution, environment lookup, clock access, networking, printing, or hidden global state
- WHEN the core purity rail runs
- THEN validation MUST fail before the package is accepted.

### Requirement: External evaluation remains a thin shell

r[octet.nickel_export_helpers.repository_extraction.imperative_shell] The standalone CLI MUST keep argument parsing, bounded file reads, explicit dependency intake, external Nickel invocation, generated-file writes, stdout or stderr rendering, and process exits in a thin imperative shell.

#### Scenario: Undeclared import blocks acceptance

r[octet.nickel_export_helpers.repository_extraction.imperative_shell.undeclared_import]
- GIVEN evaluator output depends on an import absent from the declared or observed dependency closure
- WHEN the shell prepares core acceptance input
- THEN it MUST reject the export or add the observed import explicitly before receipt construction
- AND it MUST NOT silently accept an incomplete source identity.

### Requirement: Canonical manifests and receipts bind exact identity

r[octet.nickel_export_helpers.repository_extraction.canonical_contract] The standalone contract MUST use versioned schemas and BLAKE3 identities to bind source bytes, dependency bytes, output bytes, evaluator identity, normalized options, format, selector or contract metadata, destination label, and required non-claims.

#### Scenario: Equivalent inputs produce stable records

r[octet.nickel_export_helpers.repository_extraction.canonical_contract.stable]
- GIVEN identical declared inputs, exact bytes, evaluator descriptor, options, and schema version
- WHEN canonical manifest and receipt construction runs twice
- THEN the serialized compatibility bytes and BLAKE3 identities MUST be identical.

#### Scenario: Legacy schemas require explicit projection

r[octet.nickel_export_helpers.repository_extraction.canonical_contract.legacy_projection]
- GIVEN Octet or Mantle must preserve a legacy manifest or receipt schema during migration
- WHEN it emits that schema from a canonical record
- THEN a versioned tested projection MUST perform the conversion
- AND the standalone implementation MUST NOT silently reuse a legacy schema name for different semantics.

### Requirement: Conformance includes positive and negative fixtures

r[octet.nickel_export_helpers.repository_extraction.conformance] The standalone repository MUST maintain a cross-consumer conformance corpus with positive and negative cases for external and embedded evaluator integrations.

#### Scenario: Supported export profiles pass

r[octet.nickel_export_helpers.repository_extraction.conformance.positive]
- GIVEN reviewed JSON, TOML, YAML, raw, selector, contract, multiple-dependency, external-evaluator, and embedded-evaluator fixtures
- WHEN standalone conformance validation runs
- THEN accepted outputs, manifests, receipts, diagnostics, and legacy projections MUST match their reviewed expectations.

#### Scenario: Drift and unsafe inputs fail closed

r[octet.nickel_export_helpers.repository_extraction.conformance.negative]
- GIVEN stale source, dependency, output, evaluator, or command identity; an undeclared dependency; duplicate output; mixed evaluator; unsupported schema or format; path escape; malformed manifest; contract failure; secret-like material; or weakened non-claims
- WHEN standalone conformance validation runs
- THEN it MUST fail with deterministic diagnostics naming each applicable failure class
- AND it MUST NOT emit an accepted receipt.

### Requirement: Octet pins one canonical source

r[octet.nickel_export_helpers.repository_extraction.exact_source] Octet MUST bind its standalone dependency and cutover evidence to one immutable source coordinate and exact revision, and its dependency manifest and lockfile MUST agree on that identity.

#### Scenario: Floating or duplicate canonical source fails

r[octet.nickel_export_helpers.repository_extraction.exact_source.rejects_drift]
- GIVEN the dependency is branch-only, tag-only, a release-local override, lock-drifted, or duplicated under the canonical package identity from a different source
- WHEN Octet validates extraction cutover readiness
- THEN cutover MUST fail and name the conflicting or non-immutable source identity.

### Requirement: Octet retains only a compatibility facade

r[octet.nickel_export_helpers.repository_extraction.octet_cutover] Octet MUST delegate evaluator-neutral request, manifest, receipt, diagnostic, and freshness semantics to the pinned standalone core and MUST NOT retain an independently evolving canonical implementation.

#### Scenario: Existing Octet command delegates without semantic fork

r[octet.nickel_export_helpers.repository_extraction.octet_cutover.delegates]
- GIVEN an Octet Nickel export manifest command receives source and generated artifact facts
- WHEN it validates or renders compatibility output
- THEN Octet MUST call the standalone semantic core or an explicit projection adapter
- AND filesystem and command behavior MUST remain in the Octet shell.

### Requirement: Runtime checks remain Nickel-free

r[octet.nickel_export_helpers.repository_extraction.runtime_boundary] The extraction MUST preserve the boundary that ordinary Octet runtime checks consume deterministic generated artifacts and do not dynamically evaluate Nickel.

#### Scenario: Ordinary check has no evaluator authority

r[octet.nickel_export_helpers.repository_extraction.runtime_boundary.no_dynamic_eval]
- GIVEN generated policy artifacts and their accepted freshness evidence already exist
- WHEN ordinary `cargo octet check` runs
- THEN it MUST NOT require a Nickel binary, embedded evaluator, source write, or export-time secret material.

### Requirement: Downstream cutovers remain consumer-owned

r[octet.nickel_export_helpers.repository_extraction.consumer_handoff] Mantle, Cairn, Trellis, and Animus MUST adopt the standalone repository only through reviewable consumer-owned Cairn changes with exact pins, compatibility fixtures, rollback boundaries, and retained-authority inventories.

#### Scenario: Octet extraction does not mutate consumers implicitly

r[octet.nickel_export_helpers.repository_extraction.consumer_handoff.separate_changes]
- GIVEN the standalone repository and Octet cutover are ready
- WHEN downstream migration is planned
- THEN each consumer MUST own its dependency and adapter change in its repository
- AND no workspace-relative path or unreviewed cross-repository edit may be treated as durable product behavior.

### Requirement: Extraction validation remains bounded

r[octet.nickel_export_helpers.repository_extraction.validation] Extraction validation MUST prove declared export identity, deterministic contract behavior, exact dependency ownership, and selected conformance compatibility only.

#### Scenario: Passing receipt avoids overclaims

r[octet.nickel_export_helpers.repository_extraction.validation.non_claims]
- GIVEN standalone and Octet cutover validation passes
- WHEN an operator reviews the result
- THEN it MUST state that the evidence does not prove Nickel evaluator correctness, policy semantics, deployability, build success, arbitrary consumer compatibility, complete secret absence, or release eligibility.
