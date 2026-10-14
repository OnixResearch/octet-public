# Dependency Trust Specification

## Purpose

Adds a reusable source-boundary core for dependency and provenance evidence.

## Requirements

### Requirement: Source boundary core contract
r[octet.dependency_trust.source_boundary_core.contract] Octet MUST define pure source-boundary contracts for source kind, revision identity, dirty-state policy, vendored snapshot metadata, registry package metadata, generated artifact metadata, digest inventories, and required non-claims.

#### Scenario: Valid source descriptor validates
r[octet.dependency_trust.source_boundary_core.fixtures.positive]
- GIVEN a source descriptor has a supported source kind, required revision or package identity, selected-path policy, digest inventory, dirty-state disposition, and non-claim boundary
- WHEN source-boundary validation runs
- THEN validation MUST accept the descriptor and report its normalized source identity.

#### Scenario: Invalid source descriptor fails closed
r[octet.dependency_trust.source_boundary_core.fixtures.negative]
- GIVEN a source descriptor has missing revision metadata, disallowed dirty state, path escape, missing digest inventory, stale vendored digest, unknown source kind, missing non-claims, or dependency-trust overclaim
- WHEN source-boundary validation runs
- THEN validation MUST fail with deterministic diagnostics naming the invalid descriptor field.

### Requirement: Vendored snapshots are explicit source boundaries
r[octet.dependency_trust.source_boundary_core.vendored] Octet MUST require vendored source descriptors to bind upstream source identity, revision, selected paths, digest inventory, and local-patch disposition.

#### Scenario: Vendored snapshot binds upstream revision
r[octet.dependency_trust.source_boundary_core.vendored.revision]
- GIVEN a source descriptor identifies a vendored snapshot
- WHEN validation runs
- THEN the descriptor MUST name the upstream source identity, imported revision, selected path policy, digest inventory, and local-patch disposition.

### Requirement: Source-boundary core is pure
r[octet.dependency_trust.source_boundary_core.validation] Source-boundary validation MUST be pure deterministic logic over in-memory descriptors and inventories.

#### Scenario: Shell owns source discovery
r[octet.dependency_trust.source_boundary_core.validation.shell]
- GIVEN Octet gathers source facts from Git, Cargo metadata, local worktrees, vendored directories, generated artifacts, or registries
- WHEN source-boundary validation is invoked
- THEN filesystem traversal, Git commands, registry fetches, Cargo metadata inspection, stdout/stderr, and receipt writing MUST remain outside the source-boundary core.

### Requirement: Source-boundary adapters preserve existing receipts
r[octet.dependency_trust.source_boundary_core.adapters] Octet SHOULD provide adapters for existing cross-repo source acquisition and dependency-trust receipts without changing their public field names or trust boundaries.

#### Scenario: Existing source receipt adapts
r[octet.dependency_trust.source_boundary_core.adapters.receipt]
- GIVEN an existing Octet source acquisition or dependency-trust receipt uses current public source fields
- WHEN an adapter validates the receipt
- THEN the adapter SHOULD preserve those fields while routing reusable source-boundary checks through the shared core.

### Requirement: Source-boundary core remains bounded
r[octet.dependency_trust.source_boundary_core.docs] Passing source-boundary validation MUST NOT prove dependency trust, code correctness, supply-chain safety, release eligibility, verifier soundness, or downstream behavioral correctness.

#### Scenario: Boundary is visible
r[octet.dependency_trust.source_boundary_core.final_validation]
- GIVEN a source-boundary validation report passes
- WHEN an operator reads the supported claim
- THEN the report MUST state that validation proves only source descriptor completeness, identity linkage, digest inventory consistency, and non-claim conformance.
