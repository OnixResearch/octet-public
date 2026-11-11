## ADDED Requirements

### Requirement: Generated Policy Artifacts [r[generated-policy-artifacts]]

The system MUST export deterministic generated artifacts from the validated Tiger Style policy registry for consumer checkers and CI review.

The artifact bundle MUST include `tigerstyle-policies.json`, `tigerstyle-check-config.toml`, and `tigerstyle-seed-manifests.json`.

#### Scenario: Artifact export writes named files [r[generated-policy-artifacts.scenario.named-files]]

- GIVEN a registry JSON exported from `standards/policies/registry.ncl`
- WHEN the standards artifact export command writes to an output directory
- THEN the output directory contains `tigerstyle-policies.json`
- AND the output directory contains `tigerstyle-check-config.toml`
- AND the output directory contains `tigerstyle-seed-manifests.json`

#### Scenario: Artifact-backed checker run [r[generated-policy-artifacts.scenario.checker-consumes-artifacts]]

- GIVEN a generated artifact directory
- WHEN `tigerstyle-standards check` runs with that directory and a structural fixture
- THEN the checker loads the policy registry from `tigerstyle-policies.json`
- AND the checker emits the same policy diagnostics and enforcement metadata as the registry-backed path

#### Scenario: Check config summarizes enforcement seams [r[generated-policy-artifacts.scenario.check-config-summary]]

- GIVEN generated policy artifacts
- WHEN the check config artifact is inspected
- THEN it lists checker IDs, policy families, hard policy IDs, structural policy IDs, and verified policy IDs in deterministic order
