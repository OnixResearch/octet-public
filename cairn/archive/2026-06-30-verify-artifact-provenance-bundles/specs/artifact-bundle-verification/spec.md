# Artifact Bundle Verification Specification

## Purpose

Defines the `artifact-bundle-verification` capability.

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
