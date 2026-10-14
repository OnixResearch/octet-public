## ADDED Requirements

### Requirement: Mutating input in pure lint

r[octet.purity_effect_lints.mutating_input_in_pure] Octet MUST provide a `mutating_input_in_pure` lint that flags function parameters capable of mutating caller-owned state in declared pure contexts.

#### Scenario: Mutating input capability is flagged

r[octet.purity_effect_lints.mutating_input_in_pure.flags_mutating_input]

- GIVEN a function is in a configured pure/core context
- WHEN its signature accepts `&mut`, raw pointer parameters, or parameters typed as locks, atomics, cells, or unsafe cells
- THEN `mutating_input_in_pure` MUST emit a diagnostic at the hazardous parameter.
- AND the diagnostic MUST recommend an owned state-transition input/output or moving the mutation to an imperative shell.

#### Scenario: Explicit state transition stays clean

r[octet.purity_effect_lints.mutating_input_in_pure.allows_explicit_transition]

- GIVEN a function receives owned state, borrows immutable input, mutates only local temporaries, or lives in a shell scope
- WHEN the lint runs
- THEN `mutating_input_in_pure` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.mutating_input_in_pure.catalog]

- GIVEN `mutating_input_in_pure` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
