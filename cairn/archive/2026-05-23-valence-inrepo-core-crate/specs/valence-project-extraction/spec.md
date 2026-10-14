# Valence Project Extraction Delta

## ADDED Requirements

### Requirement: Phase 1 in-repository core crate seam
r[octet.valence_project_extraction.phase1.inrepo_core_crate] The system MUST provide an in-repository `valence-core` workspace crate for pure deterministic Valence evidence logic before any standalone repository cutover.

#### Scenario: Core crate is workspace-owned
r[octet.valence_project_extraction.phase1.inrepo_core_crate.workspace]
- GIVEN Octet remains the repository and CLI shell during Phase 1
- WHEN the workspace manifest is inspected
- THEN `valence-core` is a workspace member and can be tested independently.

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
