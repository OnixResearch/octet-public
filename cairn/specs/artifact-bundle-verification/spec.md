# Artifact Bundle Verification Specification

## Purpose

Defines the `artifact-bundle-verification` capability, including capability-bounded local filesystem roots for Octet runner, cache, proof, source, and artifact bundle shell boundaries.

## Requirements

### Requirement: Artifact bundle verifier command

r[octet.artifact_bundle_verification.command] Octet SHOULD provide a local verifier command for saved `cargo octet check --artifact-dir` bundles that performs validation without mutating the artifact directory.

#### Scenario: Operator verifies a saved bundle

r[octet.artifact_bundle_verification.command.local]

- GIVEN an operator has a saved Octet artifact directory
- WHEN they run the artifact bundle verifier for that directory
- THEN the command SHOULD read the bundle locally and report whether the bundle is valid.
- AND the command MUST NOT rewrite, delete, or create files inside the artifact directory.

#### Scenario: Deterministic verifier output

r[octet.artifact_bundle_verification.command.deterministic_output]

- GIVEN the same valid artifact directory and verifier options
- WHEN the verifier runs more than once
- THEN its machine-readable output MUST list artifacts, source IDs, checker IDs, evidence tiers, BLAKE3 hashes, checked scopes, and manual-review actions in deterministic order.

### Requirement: Complete bundle provenance coverage

r[octet.artifact_bundle_verification.complete_bundle] The verifier MUST require every expected non-provenance artifact to have exactly one matching provenance record with the expected BLAKE3 content hash, checker ID, evidence tier, artifact locator, and checked scope.

#### Scenario: Complete bundle passes

r[octet.artifact_bundle_verification.complete_bundle.passes]

- GIVEN a bundle contains `command.txt`, `status.json`, `summary.txt`, `provenance.jsonl`, and the structured output expected for its output format
- WHEN every non-provenance artifact has exactly one provenance record with a matching BLAKE3 hash and accepted checker ID
- THEN verification MUST pass.

#### Scenario: Provenance coverage is exact

r[octet.artifact_bundle_verification.complete_bundle.exact]

- GIVEN a bundle contains extra provenance records or an artifact without provenance
- WHEN verification runs
- THEN verification MUST fail with diagnostics naming the uncovered artifact or unmatched provenance record.

### Requirement: Artifact bundle verification fails closed

r[octet.artifact_bundle_verification.fail_closed] The verifier MUST fail closed for partial bundles, stale extra artifacts, malformed provenance, hash mismatches, unknown checker IDs, invalid evidence tiers, and missing manual-review actions.

#### Scenario: Partial or stale bundle fails

r[octet.artifact_bundle_verification.fail_closed.file_set]

- GIVEN an artifact directory is missing an expected artifact or contains an unexpected sidecar
- WHEN verification runs
- THEN verification MUST fail with a diagnostic naming the missing or unexpected file.

#### Scenario: Inconsistent provenance fails

r[octet.artifact_bundle_verification.fail_closed.provenance]

- GIVEN `provenance.jsonl` contains malformed JSON, an unknown checker ID, an invalid evidence tier, a missing manual-review action, or a BLAKE3 hash mismatch
- WHEN verification runs
- THEN verification MUST fail with a diagnostic naming the offending record and field.

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
