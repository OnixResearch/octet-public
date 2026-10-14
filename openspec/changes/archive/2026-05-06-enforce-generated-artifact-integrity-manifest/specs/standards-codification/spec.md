## ADDED Requirements

### Requirement: Generated Artifact Integrity Manifest [r[generated-artifact-integrity-manifest]]

The system MUST export a generated artifact integrity manifest for policy artifact directories so downstream consumers can validate the complete generated bundle before consuming policy metadata.

#### Scenario: Artifact export writes integrity manifest [r[generated-artifact-integrity-manifest.scenario.export-writes-manifest]]

- GIVEN a policy registry exported through `tigerstyle-standards export-artifacts`
- WHEN the generated artifact directory is written
- THEN the output directory contains `tigerstyle-artifact-manifest.json`
- AND the manifest declares schema version `tigerstyle-artifact-manifest/v1`
- AND the manifest records the BLAKE3 digest and byte length for every generated artifact file except itself

### Requirement: Artifact-Backed Integrity Validation [r[artifact-backed-integrity-validation]]

The system MUST validate the generated artifact integrity manifest before artifact-backed standards checks consume a policy artifact directory.

#### Scenario: Artifact-backed check rejects tampered generated artifact [r[artifact-backed-integrity-validation.scenario.rejects-tampered-artifact]]

- GIVEN a generated policy artifact directory with a valid manifest
- AND one generated artifact file is edited after export
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs
- THEN the command exits non-zero
- AND the diagnostic names the artifact integrity manifest mismatch

#### Scenario: Artifact-backed check accepts intact generated bundle [r[artifact-backed-integrity-validation.scenario.accepts-intact-bundle]]

- GIVEN a generated policy artifact directory with an intact manifest and generated artifact files
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs
- THEN the manifest validation succeeds
- AND no artifact integrity diagnostic is emitted
