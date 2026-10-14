## ADDED Requirements

### Requirement: Nondeterministic output lint

r[octet.purity_effect_lints.nondeterministic_output] Octet SHOULD provide a `nondeterministic_output` lint that flags obvious unordered hash-collection iteration flowing into ordered outputs without a stable sort.

#### Scenario: Unsorted hash-derived ordered output is flagged

r[octet.purity_effect_lints.nondeterministic_output.flags_unsorted_hash_output]

- GIVEN Rust code iterates a `HashMap` or `HashSet`
- WHEN the iterated values are directly collected, joined, serialized, or rendered into an ordered output without a stable sort
- THEN `nondeterministic_output` SHOULD emit a diagnostic at the ordered-output construction.
- AND the diagnostic MUST recommend sorting by a stable key or using an ordered collection type.

#### Scenario: Stable ordering stays clean

r[octet.purity_effect_lints.nondeterministic_output.allows_stable_order]

- GIVEN code sorts hash-derived values before ordered output, uses `BTreeMap` or `BTreeSet`, or returns an explicitly unordered collection
- WHEN the lint runs
- THEN `nondeterministic_output` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.nondeterministic_output.catalog]

- GIVEN `nondeterministic_output` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
