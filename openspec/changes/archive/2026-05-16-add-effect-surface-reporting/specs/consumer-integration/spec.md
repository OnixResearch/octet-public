## ADDED Requirements

### Requirement: Effect Surface Reporting [r[effect-surface-reporting]]

The system MUST classify Slotcar findings into reviewable effect families such as `time`, `failure`, `resource`, `concurrency`, `io`, `remote`, `randomness`, `unsafe`, or `unknown`. Machine outputs MUST include the effect family, and human summaries MUST aggregate findings by effect family.

#### Scenario: JSON findings include effect family [r[effect-surface-reporting.scenario.json-findings-include-effect-family]]

- GIVEN JSON output is requested for findings from safety, purity, and concurrency lints
- WHEN `cargo slotcar check --output-format json` emits findings
- THEN each finding includes an `effect_family` field resolved from static diagnostic metadata

#### Scenario: Human summary aggregates effects [r[effect-surface-reporting.scenario.human-summary-aggregates-effects]]

- GIVEN a run emits findings across multiple effect families
- WHEN the check summary is printed
- THEN the summary includes deterministic per-effect counts sorted by count then family name

#### Scenario: Unknown lints stay explicit [r[effect-surface-reporting.scenario.unknown-lints-stay-explicit]]

- GIVEN a diagnostic is emitted for a lint missing effect metadata
- WHEN machine output is generated
- THEN the finding uses `effect_family: "unknown"` and the metadata coverage check fails in repo tests

