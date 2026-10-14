## ADDED Requirements

### Requirement: Ambient environment lint

r[octet.purity_effect_lints.ambient_env] Octet MUST provide an `ambient_env` lint that flags direct production reads or mutations of process environment state and guides maintainers toward explicit input parameters or shell-boundary capture.

#### Scenario: Ambient environment access is flagged

r[octet.purity_effect_lints.ambient_env.flags_env]

- GIVEN production Rust code directly calls a known environment API such as env var reads, process argument reads, current-directory reads, or environment mutation
- WHEN the lint runs
- THEN `ambient_env` MUST emit a diagnostic at the environment access.
- AND the diagnostic MUST explain that environment state is hidden input or output.

#### Scenario: Explicit input and test setup stay clean

r[octet.purity_effect_lints.ambient_env.allows_explicit_input]

- GIVEN code receives configuration, paths, or arguments as explicit parameters, or the access occurs in a recognized test context or justified shell helper
- WHEN the lint runs
- THEN `ambient_env` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.ambient_env.catalog]

- GIVEN `ambient_env` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
