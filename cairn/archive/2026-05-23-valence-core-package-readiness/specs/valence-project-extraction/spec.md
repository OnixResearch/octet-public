# Valence Project Extraction Specification Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
