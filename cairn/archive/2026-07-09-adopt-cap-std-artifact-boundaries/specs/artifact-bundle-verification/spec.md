# Artifact Bundle Verification Specification

## Purpose

Adds capability-bounded local filesystem roots for Octet runner, cache, proof, source, and artifact bundle shell boundaries.

## Requirements

### Requirement: Octet artifact filesystem authority is capability-rooted
r[octet.artifact_bundle_verification.cap_std_boundary.root_wrappers] Octet MUST represent workspace, source, artifact, proof, and cache filesystem authority as typed capability roots opened by runner or verifier shells.

#### Scenario: Valid artifact path stays under the declared root
r[octet.artifact_bundle_verification.cap_std_boundary.tests.positive]
- GIVEN Octet has opened an operator-declared workspace, source, artifact, proof, or cache root
- WHEN runner or verifier code reads, writes, verifies, or caches a valid relative path through the typed root
- THEN the operation MUST remain confined to that declared root while preserving existing artifact formats, provenance records, and valid-input outputs.

#### Scenario: Invalid artifact path cannot escape the root
r[octet.artifact_bundle_verification.cap_std_boundary.tests.negative]
- GIVEN input names `../` traversal, an absolute path, a missing capability root, a symlink escape attempt, or a cache/output escape attempt
- WHEN Octet resolves the path through the typed root
- THEN Octet MUST fail closed before reading, writing, verifying, or caching outside the declared root.

### Requirement: Capability dependency stays outside analysis cores
r[octet.artifact_bundle_verification.cap_std_boundary.dependency] Octet MUST add `cap-std` only to runner or shell crates/modules that own filesystem effects.

#### Scenario: Lint and standards cores stay filesystem-neutral
r[octet.artifact_bundle_verification.cap_std_boundary.conversion]
- GIVEN lint analysis cores, standards validators, policy validators, and Valence DTO construction are reviewed
- WHEN cap-std adoption is complete
- THEN those cores MUST operate on in-memory source text, artifact bytes, policy records, execution plans, or validated relative locators and MUST NOT depend on `cap-std` or open ambient filesystem paths directly.

### Requirement: Capability boundary is documented without checked-path overclaims
r[octet.artifact_bundle_verification.cap_std_boundary.docs] Octet MUST document that capability roots bound local filesystem authority only and do not prove lint soundness, source correctness, proof validity, downstream release eligibility, or whole-program safety.

#### Scenario: Closeout evidence includes artifact boundary validation
r[octet.artifact_bundle_verification.cap_std_boundary.validation]
- GIVEN the cap-std artifact-boundary change is ready to archive
- WHEN focused runner, artifact-verifier, cache, source-acquisition, Cairn validation, and gate checks run
- THEN the evidence MUST include positive and negative filesystem-boundary coverage and visible checked-path non-claim documentation.
