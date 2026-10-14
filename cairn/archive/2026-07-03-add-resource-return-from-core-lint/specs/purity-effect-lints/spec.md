## ADDED Requirements

### Requirement: Resource return from core lint

r[octet.purity_effect_lints.resource_return_from_core] Octet MUST provide a `resource_return_from_core` lint that flags functions in configured core source scopes whose return types contain concrete resource or capability handles.

#### Scenario: Core resource return is flagged

r[octet.purity_effect_lints.resource_return_from_core.flags_resource_return]

- GIVEN a function is declared inside a configured core source scope
- WHEN its return type contains a known file, socket, process, command, task, channel, connection, or external-client handle
- THEN `resource_return_from_core` MUST emit a diagnostic at the return type.
- AND the diagnostic MUST recommend returning domain values, command descriptions, or plans that a shell executes.

#### Scenario: Descriptions and shell resources stay clean

r[octet.purity_effect_lints.resource_return_from_core.allows_descriptions]

- GIVEN shell code returns resource handles or core code returns command-description enums, serializable plans, domain values, or port-independent data
- WHEN the lint runs
- THEN `resource_return_from_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.resource_return_from_core.catalog]

- GIVEN `resource_return_from_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
