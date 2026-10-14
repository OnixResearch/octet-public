## ADDED Requirements

### Requirement: Logging in core lint

r[octet.purity_effect_lints.logging_in_core] Octet MUST provide a `logging_in_core` lint that flags direct printing, debugging, logging, tracing, or metrics emission inside configured core source scopes.

#### Scenario: Core observability side effect is flagged

r[octet.purity_effect_lints.logging_in_core.flags_logging]

- GIVEN a Rust source file is inside a configured core source scope
- WHEN it directly prints, debugs, logs, traces, or emits metrics through a known macro or call path
- THEN `logging_in_core` MUST emit a diagnostic at the user-written site.
- AND the diagnostic MUST recommend returning structured events or using a shell-owned observability port.

#### Scenario: Returned events and shell logging stay clean

r[octet.purity_effect_lints.logging_in_core.allows_events]

- GIVEN core code returns structured events or shell code records logs/metrics
- WHEN the lint runs
- THEN `logging_in_core` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.logging_in_core.catalog]

- GIVEN `logging_in_core` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
