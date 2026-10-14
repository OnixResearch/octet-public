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

## ADDED Requirements

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
