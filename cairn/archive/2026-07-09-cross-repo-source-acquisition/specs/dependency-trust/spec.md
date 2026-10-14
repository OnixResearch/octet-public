# Dependency Trust Specification

## Purpose

Adds source-acquisition receipts for stack-owned dependency and input provenance.

## Requirements

### Requirement: Source-acquisition receipts
r[octet.dependency_trust.source_acquisition_receipts] Octet MUST support source-acquisition receipts that classify registry, public Git, private SSH Git, local path, vendored snapshot, Nix flake, and other stack-owned dependency sources with explicit identity and non-claim boundaries.

#### Scenario: Approved source rows pass
r[octet.dependency_trust.source_acquisition_receipts.positive]
- GIVEN a source row names a package or input label, source kind, repository or URL, revision or lock identity, credential boundary, release eligibility, local override policy, and required non-claims
- WHEN Octet validates the source-acquisition receipt
- THEN validation MUST pass and preserve the source kind for evidence summaries.

#### Scenario: Invalid source rows fail closed
r[octet.dependency_trust.source_acquisition_receipts.negative]
- GIVEN a source row is unpinned, path-only while marked release-eligible, private SSH without credential boundary, vendored without source revision metadata, stale against the lock identity, or missing non-claims
- WHEN Octet validates the source-acquisition receipt
- THEN validation MUST fail with deterministic diagnostics naming the source row.

### Requirement: Source-acquisition validation
r[octet.dependency_trust.source_acquisition_receipts.validation] Octet MUST validate source-acquisition receipts through a pure decision core fed by shell-extracted Cargo, Nix, lockfile, and optional manifest metadata.

#### Scenario: Shell extracts metadata only
r[octet.dependency_trust.source_acquisition_receipts.validation.shell]
- GIVEN Cargo or Nix source metadata must be inspected
- WHEN source-acquisition validation runs
- THEN the shell MUST read files and extract metadata before passing parsed rows to the pure checker.

#### Scenario: Pure checker reports policy result
r[octet.dependency_trust.source_acquisition_receipts.validation.core]
- GIVEN parsed source rows and policy settings
- WHEN the pure checker evaluates them
- THEN it MUST return pass/fail diagnostics without performing filesystem, network, process, or environment access.

### Requirement: Evidence separation
r[octet.dependency_trust.source_acquisition_receipts.evidence_separation] Source-acquisition receipts MUST remain separate from vulnerability scans, license checks, dependency trust audits, and formal proof evidence.

#### Scenario: Source receipt remains policy evidence
r[octet.dependency_trust.source_acquisition_receipts.evidence_separation.tier]
- GIVEN a source-acquisition receipt validates a private SSH dependency or vendored source snapshot
- WHEN Octet emits provenance
- THEN the evidence tier MUST remain policy/source validation or manual-review as configured, not dependency-audit, cargo-vet trust, or formal proof.

#### Scenario: Parent link is allowed without tier merging
r[octet.dependency_trust.source_acquisition_receipts.evidence_separation.parent]
- GIVEN a dependency-trust receipt cites source-acquisition evidence as a parent artifact
- WHEN artifact verification runs
- THEN Octet MUST verify the parent link without merging the source-acquisition evidence tier into the dependency-trust tier.

### Requirement: Consumer guidance
r[octet.dependency_trust.source_acquisition_receipts.docs] Octet documentation MUST explain how consumers should record public Git, private SSH, vendored, and local sibling override dependencies for stack-owned repos.

#### Scenario: Private SSH boundary is visible
r[octet.dependency_trust.source_acquisition_receipts.docs.ssh]
- GIVEN a consumer uses a private SSH stack dependency
- WHEN they create source-acquisition evidence
- THEN guidance MUST require the credential boundary and release eligibility status to be explicit.

#### Scenario: Local override is non-release by default
r[octet.dependency_trust.source_acquisition_receipts.docs.local]
- GIVEN a consumer uses a local sibling path override
- WHEN they create source-acquisition evidence
- THEN guidance MUST state that the override is development-only unless a separate release source identity is declared.

### Requirement: Final validation evidence
r[octet.dependency_trust.source_acquisition_receipts.final_validation] The change MUST include positive and negative fixtures, artifact verification, and Cairn validation evidence before archive.

#### Scenario: Fixture matrix covers source classes
r[octet.dependency_trust.source_acquisition_receipts.final_validation.fixtures]
- GIVEN positive and negative source-acquisition fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, and saved evidence MUST bind policy, fixture, and receipt identities.
