# Valence Project Extraction Specification

## Purpose

Define phased requirements for preparing Valence to become a standalone project while preserving Octet's current bounded evidence claims and avoiding premature repository or release cutover.

## Requirements

### Requirement: Phase 0 extraction inventory
r[octet.valence_project_extraction.phase0.inventory] The system MUST record a reviewed Phase 0 inventory of candidate Valence-owned surfaces, Octet-retained surfaces, shared handoff surfaces, and current verification entrypoints before any code or repository cutover occurs.

#### Scenario: Inventory records current surfaces
r[octet.valence_project_extraction.phase0.inventory.current]
- GIVEN the current Octet repository contains Valence-oriented code, fixtures, docs, Cairn specs, and Nix gates
- WHEN the Phase 0 extraction package is reviewed
- THEN it provides an inventory artifact that names the observed path families and verification entrypoints.

### Requirement: Extraction boundary remains planning-only
r[octet.valence_project_extraction.phase0.boundary] The system MUST classify extraction surfaces without moving code, renaming crates, changing package identity, creating a standalone repository, or cutting a release during Phase 0.

#### Scenario: Boundary avoids cutover side effects
r[octet.valence_project_extraction.phase0.boundary.planning_only]
- GIVEN Phase 0 is an extraction-readiness planning package
- WHEN the package is validated
- THEN it does not require code movement, crate renames, repository creation, tag movement, or release publication.

### Requirement: Handoff gates are explicit
r[octet.valence_project_extraction.phase0.gates] The system MUST define the named Nix and Cairn gates that a future Valence extraction must preserve before any cutover can be accepted.

#### Scenario: Gate contract names release and aggregate rails
r[octet.valence_project_extraction.phase0.gates.named]
- GIVEN the current Octet repository exposes Valence evidence release and aggregate gates
- WHEN the Phase 0 handoff contract is inspected
- THEN it names the release evidence gate, aggregate object evidence gate, Cairn validation gate, and current evidence-index checker invocation or an explicit compatibility successor.

### Requirement: Extraction plan preserves bounded claims
r[octet.valence_project_extraction.phase0.non_claims] The system MUST preserve the existing bounded non-claim language when describing Valence extraction readiness.

#### Scenario: Non-claims stay explicit
r[octet.valence_project_extraction.phase0.non_claims.explicit]
- GIVEN the current Valence evidence is selected-corpus and selected-fixture replay evidence
- WHEN Phase 0 readiness is described
- THEN it does not claim correctness, semantic equivalence, exhaustive Rust coverage, general HIR/THIR/MIR extraction, public unattended production readiness, cross-machine determinism, or cross-toolchain determinism.

### Requirement: Evidence-index core moves behind the Valence crate boundary
r[octet.valence_project_extraction.phase1.evidence_index_core] The system MUST move the deterministic evidence-index render/check core into `valence-core` while preserving the existing generated Markdown contract and fail-closed issue semantics.

#### Scenario: Evidence-index core remains deterministic
r[octet.valence_project_extraction.phase1.evidence_index_core.deterministic]
- GIVEN the evidence-index renderer receives a capture time and payload commit
- WHEN `valence-core` renders and checks the page text
- THEN the roundtrip passes and negative stale/missing/weakened inputs fail closed with typed issue codes.

### Requirement: Cargo Octet remains the imperative shell
r[octet.valence_project_extraction.phase1.cli_shell] The system MUST keep filesystem access, argument parsing, stdout/stderr, and command exit behavior in `cargo-octet` while delegating pure evidence-index behavior to `valence-core`.

#### Scenario: CLI shell delegates to core
r[octet.valence_project_extraction.phase1.cli_shell.delegates]
- GIVEN an operator runs `cargo-octet evidence-index check --page docs/current-head-valence-evidence.md`
- WHEN the command validates the checked-in page
- THEN it emits the existing `valence_evidence_index.valid: PASS` and `valence_evidence_index.issue_count=0` pass lines.

### Requirement: Phase 1 preserves existing Valence gates
r[octet.valence_project_extraction.phase1.gates] The system MUST preserve the named Valence evidence Nix gates and Cairn validation after introducing the in-repo core crate.

#### Scenario: Named gates still pass
r[octet.valence_project_extraction.phase1.gates.named]
- GIVEN the in-repo Valence crate seam has been added
- WHEN focused Rust tests, `valence-evidence-index`, `valence-evidence-release`, `valence-object-evidence`, and Cairn validation run
- THEN they pass without weakening the existing bounded evidence contracts.

### Requirement: Phase 1 remains bounded and non-cutover
r[octet.valence_project_extraction.phase1.non_claims] The system MUST describe Phase 1 as a bounded in-repo crate seam and MUST NOT claim standalone repository cutover, package publication, correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

#### Scenario: No cutover or overclaim occurs
r[octet.valence_project_extraction.phase1.non_claims.explicit]
- GIVEN Phase 1 extracts a small deterministic evidence-index core seam
- WHEN the change is reviewed
- THEN it does not create a standalone repository, publish a package, move release tags, or broaden Valence evidence claims.

### Requirement: Function Object v1 core moves behind the Valence crate boundary
r[octet.valence_project_extraction.phase2.function_object_core] The system MUST move pure Function Object v1 DTOs, canonical object hash construction, source-token caveat summaries, and pure-cache eligibility metadata into `valence-core` while preserving existing object evidence semantics.

#### Scenario: Function Object core remains deterministic
r[octet.valence_project_extraction.phase2.function_object_core.deterministic]
- GIVEN the Function Object core receives the same source-scanner facts for a selected function
- WHEN `valence-core` constructs a Function Object v1 record twice
- THEN the object hash and no-correctness boundary are stable across both constructions.

#### Scenario: Function Object core remains fail-closed for purity claims
r[octet.valence_project_extraction.phase2.function_object_core.caveated]
- GIVEN source-token caveats remain unresolved by the v1 scanner
- WHEN `valence-core` constructs the Function Object v1 record
- THEN pure-cache eligibility is blocked by caveats and the record does not claim behavioral correctness.

### Requirement: Cargo Octet remains the Function Object shell
r[octet.valence_project_extraction.phase2.octet_shell] The system MUST keep source scanning, filesystem access, argument parsing, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, and command exit behavior in `cargo-octet` while delegating pure Function Object v1 construction to `valence-core`.

#### Scenario: Object shell delegates to core
r[octet.valence_project_extraction.phase2.octet_shell.delegates]
- GIVEN an operator runs existing `cargo-octet object ...` commands
- WHEN the command needs a Function Object v1 record
- THEN `cargo-octet` supplies scanner facts to `valence-core` and preserves existing object command behavior.

### Requirement: Function Object compatibility is preserved
r[octet.valence_project_extraction.phase2.hash_compatibility] The system MUST preserve the Function Object v1 canonical hash payload and existing object evidence compatibility unless a future reviewed change explicitly updates fixtures and schema contracts.

#### Scenario: Existing object evidence gates pass
r[octet.valence_project_extraction.phase2.hash_compatibility.gates]
- GIVEN Function Object v1 core construction has moved behind the Valence crate boundary
- WHEN focused object tests, `object-corpus-gate`, and `valence-object-evidence` run
- THEN they pass without weakening existing object hash, corpus, bundle, or supplement evidence contracts.

### Requirement: Phase 2 preserves existing Valence gates
r[octet.valence_project_extraction.phase2.gates] The system MUST preserve the named Valence evidence Nix gates and Cairn validation after moving Function Object v1 core construction into `valence-core`.

#### Scenario: Named gates still pass
r[octet.valence_project_extraction.phase2.gates.named]
- GIVEN the in-repo Valence Function Object seam has been added
- WHEN focused Rust tests, `object-corpus-gate`, `valence-evidence-index`, `valence-evidence-release`, `valence-object-evidence`, and Cairn validation run
- THEN they pass without broadening the existing bounded evidence contracts.

### Requirement: Phase 2 remains bounded and non-cutover
r[octet.valence_project_extraction.phase2.non_claims] The system MUST describe Phase 2 as a bounded in-repo Function Object core seam and MUST NOT claim standalone repository cutover, package publication, correctness, semantic equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

#### Scenario: No cutover or overclaim occurs
r[octet.valence_project_extraction.phase2.non_claims.explicit]
- GIVEN Phase 2 extracts selected Function Object v1 core construction
- WHEN the change is reviewed
- THEN it does not create a standalone repository, publish a package, move release tags, or broaden Valence evidence claims.

### Requirement: Valence core carries package-consumability metadata
r[octet.valence_project_extraction.phase3.package_metadata] The system MUST add package metadata and README/API boundary documentation to `valence-core` so downstream readiness can be inspected before any standalone cutover.

#### Scenario: Metadata and README describe the boundary
r[octet.valence_project_extraction.phase3.package_metadata.boundary]
- GIVEN `valence-core` remains an in-repository crate
- WHEN its manifest, README, and crate-level docs are inspected
- THEN they name package metadata, current pure core surfaces, Octet-retained shell responsibilities, and bounded non-claims.

### Requirement: Valence core package readiness has a named Nix rail
r[octet.valence_project_extraction.phase3.package_readiness_gate] The system MUST expose a directly buildable `valence-core` package output and a named `valence-core-package-readiness` Nix check.

#### Scenario: Readiness rail proves package shape
r[octet.valence_project_extraction.phase3.package_readiness_gate.named]
- GIVEN `valence-core` has package metadata and public module boundaries
- WHEN `checks.x86_64-linux.valence-core-package-readiness` runs
- THEN it asserts the package builds, required metadata is present, public modules are exposed, and no-cutover/non-claim documentation exists.

### Requirement: Aggregate Valence evidence includes package readiness
r[octet.valence_project_extraction.phase3.aggregate_gate] The system MUST include the package-readiness rail in the aggregate `valence-object-evidence` check without removing existing evidence replay rails.

#### Scenario: Aggregate gate reports package readiness
r[octet.valence_project_extraction.phase3.aggregate_gate.pass_line]
- GIVEN the package-readiness rail is part of Valence extraction readiness
- WHEN `checks.x86_64-linux.valence-object-evidence` runs
- THEN it requires the package-readiness check and reports a package-readiness PASS line alongside existing evidence replay PASS lines.

### Requirement: Phase 3 preserves existing Valence gates
r[octet.valence_project_extraction.phase3.verification] The system MUST preserve focused `valence-core` tests, API documentation generation, package listing, named Nix gates, aggregate Valence evidence, and Cairn validation after adding package-readiness scaffolding.

#### Scenario: Focused verification remains green
r[octet.valence_project_extraction.phase3.verification.focused]
- GIVEN the package-readiness rail has been added
- WHEN focused Rust tests, `cargo doc -p valence-core --no-deps`, `cargo package -p valence-core --allow-dirty --no-verify --list`, `valence-core-package-readiness`, `valence-object-evidence`, and Cairn validation/gates run
- THEN they pass without broadening existing bounded evidence contracts.

### Requirement: Phase 3 remains bounded and non-cutover
r[octet.valence_project_extraction.phase3.non_claims] The system MUST describe Phase 3 as in-repository package-readiness scaffolding and MUST NOT claim standalone repository cutover, package publication, external consumer compatibility, correctness, semantic equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

#### Scenario: No publication or overclaim occurs
r[octet.valence_project_extraction.phase3.non_claims.explicit]
- GIVEN Phase 3 adds package-readiness metadata and a named local rail
- WHEN the change is reviewed
- THEN it does not publish a package, create a standalone repository, move release tags, transfer release ownership, or broaden Valence evidence claims.

### Requirement: Phase 1 in-repository core crate seam
r[octet.valence_project_extraction.phase1.inrepo_core_crate] The system MUST provide an in-repository `valence-core` workspace crate for pure deterministic Valence evidence logic before any standalone repository cutover.

#### Scenario: Core crate is workspace-owned
r[octet.valence_project_extraction.phase1.inrepo_core_crate.workspace]
- GIVEN Octet remains the repository and CLI shell during Phase 1
- WHEN the workspace manifest is inspected
- THEN `valence-core` is a workspace member and can be tested independently.

#### Scenario: Function Object core is Valence-owned in-repo
r[octet.valence_project_extraction.phase2.function_object_core.workspace]
- GIVEN Phase 2 extends the in-repository Valence crate seam
- WHEN Function Object v1 DTO/hash/caveat construction is inspected
- THEN the pure deterministic core is exposed from `valence-core` and can be tested independently.

#### Scenario: Core crate has a package readiness rail
r[octet.valence_project_extraction.phase3.package_readiness.workspace]
- GIVEN Phase 3 keeps `valence-core` in the Octet workspace
- WHEN the package-readiness rail is inspected
- THEN the crate has package metadata, README/API boundary documentation, and a named Nix readiness check without creating a standalone repository or publishing a package.

### Requirement: Valence core readiness inventory stays synchronized

r[octet.valence_project_extraction.phase4.readiness_inventory] The system MUST keep the `valence-core` README, crate-level documentation, and public module exports synchronized for every supported in-repository pure evidence surface.

#### Scenario: Inventory names current surfaces

r[octet.valence_project_extraction.phase4.readiness_inventory.in_sync]

- GIVEN `valence-core` exports public modules for evidence-index rendering/checking, Function Object v1 construction, and evidence-codebase indexing
- WHEN the readiness inventory is inspected
- THEN the README, crate-level docs, and public exports MUST consistently name `evidence_index`, `function_object`, and `evidence_codebase_index`.
- AND each named surface MUST state whether Octet retains CLI/filesystem/scanner/orchestration responsibilities.

#### Scenario: Inventory drift fails closed

r[octet.valence_project_extraction.phase4.readiness_inventory.fail_closed]

- GIVEN a public `valence-core` module is missing from the README or a README entry names a removed public module
- WHEN the readiness rail checks module inventory
- THEN it MUST fail with a deterministic diagnostic naming the missing or stale module.

### Requirement: Valence core readiness rail proves API shape

r[octet.valence_project_extraction.phase4.readiness_gate] The system MUST strengthen `checks.x86_64-linux.valence-core-package-readiness` so it proves the current API documentation, focused tests, package listing, package metadata, public module inventory, and boundary language for the in-repository `valence-core` seam.

#### Scenario: Named readiness rail exercises focused checks

r[octet.valence_project_extraction.phase4.readiness_gate.named]

- GIVEN `valence-core` remains an in-repository workspace crate
- WHEN `checks.x86_64-linux.valence-core-package-readiness` runs
- THEN it MUST require focused `valence-core` unit tests, API documentation generation, package listing, required package metadata, public module exposure, synchronized module inventory, and no-cutover/non-claim documentation before reporting success.

### Requirement: Phase 4 readiness remains bounded and non-cutover

r[octet.valence_project_extraction.phase4.non_claims] The system MUST describe Phase 4 readiness hardening as an in-repository package/API drift guard and MUST NOT claim standalone repository cutover, package publication, external consumer compatibility, behavioral correctness, semantic equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

#### Scenario: No cutover or overclaim occurs

r[octet.valence_project_extraction.phase4.non_claims.no_cutover]

- GIVEN the readiness rail now checks docs, tests, API docs, package listing, metadata, and module inventory
- WHEN the change is reviewed
- THEN it MUST NOT publish a package, create a standalone repository, move release tags, transfer release ownership, or broaden Valence evidence claims.

### Requirement: Function Object normalization is core-owned

r[octet.valence_project_extraction.phase4.function_object_normalization] The system MUST ensure `valence-core` owns or explicitly validates the Function Object v1 normalization contract so callers cannot silently pair raw source with normalized source from a different algorithm.

#### Scenario: Core-owned normalization is stable

r[octet.valence_project_extraction.phase4.function_object_normalization.core_owned]

- GIVEN the Function Object core receives the same selected function source and scanner facts twice
- WHEN `valence-core` constructs Function Object v1 records using the declared normalization algorithm
- THEN the normalized body hash, semantic identity, object hash, and no-correctness boundary MUST be stable across both constructions.

#### Scenario: Normalization mismatch fails closed

r[octet.valence_project_extraction.phase4.function_object_normalization.rejects_mismatch]

- GIVEN a caller provides normalized source tagged with an unsupported algorithm or inconsistent with the Function Object v1 normalization contract
- WHEN the Function Object core attempts construction
- THEN it MUST return a typed construction error and MUST NOT emit an object hash.

### Requirement: Function Object hash material is explicit

r[octet.valence_project_extraction.phase4.function_object_hash_payload] The system MUST define reviewable typed hash material for Function Object v1 object hashing and MUST preserve the existing canonical hash payload unless a future reviewed migration updates schema and fixtures.

#### Scenario: Hash payload is reviewable and compatible

r[octet.valence_project_extraction.phase4.function_object_hash_payload.reviewable]

- GIVEN a Function Object v1 record is constructed
- WHEN its object hash is computed
- THEN tests MUST assert which typed payload fields are included and which display/source-local fields are excluded.
- AND existing object corpus compatibility MUST remain unchanged unless the change includes explicit migration fixtures and release notes.

### Requirement: Function Object core API uses typed contracts

r[octet.valence_project_extraction.phase4.function_object_typed_api] The system SHOULD expose typed Rust contracts for Function Object construction errors, dependency identity statuses, effect caveat classifications, macro caveat classifications, and pure-cache blockers while preserving existing serialized Function Object v1 strings.

#### Scenario: Typed contracts serialize compatibly

r[octet.valence_project_extraction.phase4.function_object_typed_api.serializes_compatibly]

- GIVEN typed Function Object status and caveat values are used inside `valence-core`
- WHEN a Function Object v1 DTO is serialized
- THEN the serialized schema, status strings, caveat strings, pure-cache blockers, and no-correctness fields SHOULD match the existing v1 compatibility contract.
- AND malformed construction inputs SHOULD fail with typed errors rather than stringly ad hoc messages.

### Requirement: Cargo Octet remains the Function Object shell during hardening

r[octet.valence_project_extraction.phase4.function_object_shell] The system MUST keep filesystem access, source discovery, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, and process exits in `cargo-octet` while delegating Function Object normalization validation, caveat construction, pure-cache eligibility, and hash construction to `valence-core`.

#### Scenario: Shell delegates hardened core construction

r[octet.valence_project_extraction.phase4.function_object_shell.delegates]

- GIVEN an operator runs existing `cargo-octet object ...` commands
- WHEN Function Object v1 records are needed
- THEN `cargo-octet` MUST supply scanner facts to the hardened `valence-core` API and preserve existing command behavior, receipt schemas, and bounded non-claims.

### Requirement: Function Object hardening preserves evidence gates

r[octet.valence_project_extraction.phase4.function_object_gates] The system MUST preserve focused `valence-core` Function Object tests, focused `cargo-octet` object tests, object corpus gates, aggregate Valence object evidence, and Cairn validation after hardening the Function Object seam.

#### Scenario: Compatibility gates stay green

r[octet.valence_project_extraction.phase4.function_object_gates.compatibility]

- GIVEN the Function Object core API has been hardened
- WHEN focused Function Object tests, object corpus compatibility, object evidence gates, and Cairn validation run
- THEN they MUST pass without broadening existing bounded evidence contracts or claiming behavioral correctness, semantic equivalence, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

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
