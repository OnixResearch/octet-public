## ADDED Requirements

### Requirement: Impure call in core lint

r[octet.purity_effect_lints.impure_call_in_core] Octet MUST provide an `impure_call_in_core` lint that flags direct platform-effect calls inside configured `core_source_scopes` and guides maintainers toward injected ports or shell-boundary helpers.

#### Scenario: Effectful platform API is flagged in core scope

r[octet.purity_effect_lints.impure_call_in_core.flags_effect]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it directly calls a known filesystem, IO, network, process, environment, random, async-runtime, or platform-effect API
- THEN `impure_call_in_core` MUST emit a diagnostic at the effectful call site.
- AND the diagnostic MUST name the effect family and suggest injecting a port/capability or moving the call to a shell boundary.

#### Scenario: Shell boundary and injected capability stay clean

r[octet.purity_effect_lints.impure_call_in_core.allows_boundary]

- GIVEN a Rust source file is outside configured core source scopes or only calls an injected trait/capability inside a core scope
- WHEN the lint runs
- THEN `impure_call_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.impure_call_in_core.catalog]

- GIVEN `impure_call_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
