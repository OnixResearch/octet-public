## ADDED Requirements

### Requirement: Deterministic Nickel export helper

r[octet.nickel_export_helpers.export] Octet SHOULD provide a deterministic helper for exporting Nickel-authored configuration or policy artifacts into the native file format consumed by the next tool or gate.

#### Scenario: Exported bytes are stable

r[octet.nickel_export_helpers.export.stable]

- GIVEN a Nickel source file, its declared import dependencies, an optional field selector, an optional contract, an output format, and an output path
- WHEN the Nickel export helper runs twice with the same inputs and Nickel binary identity
- THEN the exported artifact bytes SHOULD be identical.
- AND the helper SHOULD support only declared formats such as JSON, TOML, YAML, or raw text.

#### Scenario: Contract failure blocks export acceptance

r[octet.nickel_export_helpers.export.contract_failure]

- GIVEN an export declares a Nickel contract for the authored configuration
- WHEN the source value fails that contract
- THEN the export MUST fail before the generated artifact is accepted as fresh.
- AND the diagnostic SHOULD name the failing source or contract field when Nickel reports it.

### Requirement: Nickel export manifest freshness

r[octet.nickel_export_helpers.manifest] Helper-produced exports MUST be covered by a deterministic BLAKE3-backed manifest that can reject stale or mixed-generation artifacts.

#### Scenario: Manifest records source and output identity

r[octet.nickel_export_helpers.manifest.records]

- GIVEN the Nickel export helper writes an artifact
- WHEN it writes or updates the export manifest
- THEN the manifest MUST record source paths, dependency paths, output format, output path, byte length, BLAKE3 output digest, generation command, and source or dependency digests.
- AND the manifest SHOULD record Nickel binary identity when available.

#### Scenario: Stale export fails verification

r[octet.nickel_export_helpers.manifest.stale]

- GIVEN a Nickel source, dependency, or generated artifact changes without refreshing the export manifest
- WHEN export freshness verification runs
- THEN verification MUST fail with a diagnostic naming the stale source, dependency, or generated artifact.

### Requirement: Nickel export boundaries

r[octet.nickel_export_helpers.boundaries] Nickel export helpers MUST preserve Octet's generation-time boundary and MUST NOT make ordinary runtime checks depend on dynamic Nickel evaluation or secret-bearing generated artifacts.

#### Scenario: Runtime runner reads generated artifacts

r[octet.nickel_export_helpers.boundaries.runtime]

- GIVEN a consumer runs ordinary `cargo octet check` with generated policy artifacts already present
- WHEN the runner loads configuration or policy inputs
- THEN the runner SHOULD consume deterministic generated artifacts rather than evaluating Nickel dynamically.
- AND a Nickel binary SHOULD only be required for explicit generation or freshness-validation commands.

#### Scenario: Secret-bearing export is excluded

r[octet.nickel_export_helpers.boundaries.secrets]

- GIVEN a Nickel-authored configuration contains raw secrets, credentials, private keys, or other secret material
- WHEN an export family is considered for manifest-backed generated artifacts
- THEN the family MUST be rejected or registered as a non-candidate with an explicit exclusion rationale.
