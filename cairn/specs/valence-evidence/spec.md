# Valence Evidence Specification

## Purpose

Defines the `valence-evidence` capability for Octet/Valence evidence closeout: generated evidence indexes, selected-corpus / selected-fixture replay gates, release-style bundle rails, and explicit non-claim boundaries for content-addressed Rust evidence.

## Requirements

### Requirement: The system
r[octet.valence_evidence.release.bundle] The system MUST expose a named local Nix check that materializes a Valence evidence release bundle with a machine-readable index, the current-head Valence evidence page, a freshness-check transcript, and BLAKE3 hashes for every indexed artifact.

#### Scenario: Release Bundle Materialized
r[octet.valence_evidence.release.bundle.materialized]
- GIVEN the checked-in current-head Valence evidence page is available in the flake source
- WHEN the operator runs the named Valence evidence release Nix check
- THEN the check produces a bundle containing `valence-evidence-release-index.json`, `current-head-valence-evidence.md`, and `evidence-index-check.txt`

#### Scenario: Release Bundle Preserves Boundaries
r[octet.valence_evidence.release.bundle.boundaries]
- GIVEN the release bundle index is generated
- WHEN the operator inspects its scope and non-claim fields
- THEN it states bounded local selected-corpus / selected-fixture evidence replay and does not claim correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction

### Requirement: The system
r[octet.valence_evidence.release.check] The system MUST fail closed if the generated release bundle index omits required artifacts, records stale artifact hashes, omits the evidence-index freshness pass signal, or weakens the bounded non-claim list.

#### Scenario: Release Bundle Hashes Verify
r[octet.valence_evidence.release.check.hashes]
- GIVEN the release bundle index lists artifacts and BLAKE3 hashes
- WHEN the release gate verifies the generated bundle
- THEN every indexed artifact hash matches the corresponding bundled file before the check succeeds

#### Scenario: Release Bundle Requires Fresh Evidence Page
r[octet.valence_evidence.release.check.freshness]
- GIVEN the release gate runs the Valence evidence-index checker
- WHEN the current-head evidence page is fresh
- THEN the bundled freshness transcript contains `valence_evidence_index.valid: PASS` and `valence_evidence_index.issue_count=0`

### Requirement: The system
r[octet.valence_evidence.release.aggregate] The system MUST compose the release evidence gate into the aggregate Valence object evidence gate.

#### Scenario: Aggregate Includes Release Gate
r[octet.valence_evidence.release.aggregate.pass]
- GIVEN the release evidence gate and existing selected Valence evidence gates pass
- WHEN the operator runs `checks.x86_64-linux.valence-object-evidence`
- THEN the aggregate gate depends on the release gate and emits a dedicated release-gate pass signal
