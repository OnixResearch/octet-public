## ADDED Requirements

### Requirement: Versioned Proof Bundle Artifacts [r[versioned-proof-bundle-artifacts]]

The system MUST export a versioned generated proof-bundle artifact for verified proof rails so downstream consumers can validate proof-contract shape before trusting proof receipts.

The proof-bundle artifact MUST include `schema_version`, `generated_from`, `registry_schema_version`, and deterministic entries that preserve policy ID, backend, command, source paths, proof paths, pass signal, trust boundaries, receipt binding, receipt provenance, and receipt verifier obligations.

#### Scenario: Proof bundle export writes versioned artifact [r[versioned-proof-bundle-artifacts.scenario.export-writes-versioned-artifact]]

- GIVEN a policy registry with at least one verified proof rail
- WHEN generated policy artifacts are exported
- THEN the output directory contains `tigerstyle-proof-bundles.json`
- AND that artifact declares `schema_version = "tigerstyle-proof-bundle/v1"`
- AND the verified rail entry preserves receipt binding, provenance, verifier, toolchain, and closure graph obligations for its policy ID

### Requirement: Proof Bundle Schema Validation [r[proof-bundle-schema-validation]]

The system MUST validate the generated proof-bundle schema when artifact-backed standards checks consume a policy artifact directory.

Artifact-backed checks MUST reject missing proof-bundle artifacts, unsupported proof-bundle schema versions, and proof-bundle entries that do not cover the verified proof rails declared by the policy artifact.

#### Scenario: Artifact-backed check rejects unsupported proof bundle schema [r[proof-bundle-schema-validation.scenario.rejects-unsupported-schema]]

- GIVEN a generated policy artifact directory whose `tigerstyle-proof-bundles.json` declares an unsupported schema version
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs against a verification fixture
- THEN the command exits non-zero
- AND the diagnostic names the unsupported proof-bundle schema version and the expected `tigerstyle-proof-bundle/v1` version

#### Scenario: Artifact-backed check accepts complete proof bundle [r[proof-bundle-schema-validation.scenario.accepts-complete-bundle]]

- GIVEN a generated policy artifact directory with a complete `tigerstyle-proof-bundles.json`
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs against a compliant verification fixture
- THEN the command exits successfully
- AND no proof-bundle schema diagnostic is emitted
