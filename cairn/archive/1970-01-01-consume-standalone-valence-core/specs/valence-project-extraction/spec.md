# Valence Project Extraction Specification

## Purpose

Adds the Octet-side cutover contract for consuming standalone Valence as the canonical `valence-core` owner.

## Requirements

### Requirement: Standalone Valence dependency manifest
r[octet.valence_project_extraction.phase5.standalone_dependency_manifest] Octet MUST define a dependency manifest that identifies the standalone Valence source used for `valence-core` semantics, the Octet-hosted modules being replaced or shimmed, and the compatibility disposition for each module.

#### Scenario: Manifest names canonical source
r[octet.valence_project_extraction.phase5.standalone_dependency_manifest.canonical_source]
- GIVEN Octet consumes Valence-owned evidence semantics
- WHEN the dependency manifest is validated
- THEN it MUST name the standalone Valence source coordinate, revision or local override policy, module rows, compatibility disposition, and required non-claim boundary.

### Requirement: Compatibility fixtures preserve v1 behavior
r[octet.valence_project_extraction.phase5.compatibility] Octet MUST prove that the standalone Valence dependency preserves Function Object v1, evidence-index, and evidence-codebase-index behavior before removing or disabling the hosted copy.

#### Scenario: Positive fixtures match
r[octet.valence_project_extraction.phase5.compatibility.positive]
- GIVEN known-good Octet fixtures for Function Object v1, evidence-index rendering, and evidence-codebase-index reports
- WHEN the standalone Valence dependency processes equivalent inputs
- THEN the fixture outputs MUST match the expected compatibility hashes and bounded non-claim text.

#### Scenario: Stale or unsupported compatibility fails closed
r[octet.valence_project_extraction.phase5.compatibility.negative]
- GIVEN a hosted module has drifted from canonical Valence without an exception, the dependency source is wrong, a fixture row is missing, or non-claims are absent
- WHEN cutover validation runs
- THEN validation MUST fail with deterministic diagnostics naming the stale, missing, or unsupported row.

### Requirement: Octet shell consumes standalone Valence
r[octet.valence_project_extraction.phase5.shell_consumes_standalone] Octet SHOULD route Valence-owned pure evidence calls through the standalone Valence dependency or an explicit compatibility shim while retaining Octet-owned shells.

#### Scenario: Shell/core boundary stays explicit
r[octet.valence_project_extraction.phase5.shell_consumes_standalone.boundary]
- GIVEN `cargo-octet` needs a Function Object, evidence-index, or evidence-codebase-index core operation
- WHEN the cutover implementation runs
- THEN Octet MUST keep filesystem access, source scanning, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, and process exits outside Valence core logic.

### Requirement: Consumer migration is documented
r[octet.valence_project_extraction.phase5.consumer_migration_docs] Octet MUST document how downstream users that depend on Octet's `valence-core` package move to standalone Valence.

#### Scenario: Aspen migration target is visible
r[octet.valence_project_extraction.phase5.consumer_migration_docs.aspen]
- GIVEN a downstream consumer currently depends on Octet git package `valence-core`
- WHEN it reads the migration guidance
- THEN the guidance MUST name the standalone Valence dependency target, the compatibility validation command, and the non-claim that cutover does not prove behavioral correctness, external consumer compatibility, or release eligibility.

### Requirement: Cutover validation remains bounded
r[octet.valence_project_extraction.phase5.cutover_validation] The cutover validation MUST prove dependency/source ownership and fixture compatibility only.

#### Scenario: Cutover report avoids overclaims
r[octet.valence_project_extraction.phase5.final_validation]
- GIVEN the cutover validation report passes
- WHEN an operator reads the supported claim
- THEN the report MUST state that the check does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility.
