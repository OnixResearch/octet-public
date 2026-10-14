## ADDED Requirements

### Requirement: Spawn in core lint

r[octet.purity_effect_lints.spawn_in_core] Octet MUST provide a `spawn_in_core` lint that flags direct thread or async task spawning inside configured core source scopes.

#### Scenario: Core spawn is flagged

r[octet.purity_effect_lints.spawn_in_core.flags_spawn]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it directly calls a known thread or async runtime spawn API
- THEN `spawn_in_core` MUST emit a diagnostic at the spawn call.
- AND the diagnostic MUST recommend returning a command/work description or using a shell-owned executor port.

#### Scenario: Shell executor ownership stays clean

r[octet.purity_effect_lints.spawn_in_core.allows_shell_executor]

- GIVEN shell code spawns work or core code returns command values or uses an injected executor abstraction without direct spawning
- WHEN the lint runs
- THEN `spawn_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.spawn_in_core.catalog]

- GIVEN `spawn_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
