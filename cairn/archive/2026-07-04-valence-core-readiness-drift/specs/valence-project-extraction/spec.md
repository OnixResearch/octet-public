# Valence Project Extraction Specification Delta

## ADDED Requirements

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
