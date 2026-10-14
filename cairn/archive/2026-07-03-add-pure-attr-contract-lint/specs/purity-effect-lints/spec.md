## ADDED Requirements

### Requirement: Pure attribute contract lint

r[octet.purity_effect_lints.pure_attr_contract] Octet MUST provide a `pure_attr_contract` lint that enforces direct-effect purity checks for functions marked with Octet's recognized pure-function marker.

#### Scenario: Marked impure function is flagged

r[octet.purity_effect_lints.pure_attr_contract.flags_marked_impurity]

- GIVEN a Rust function is marked with the recognized pure-function marker
- WHEN its signature or body directly uses a banned effect such as platform IO, ambient time/env/random, logging, spawning, global state, interior mutability, mutating input capabilities, panic paths, or resource handles
- THEN `pure_attr_contract` MUST emit a diagnostic at the violating source shape.
- AND the diagnostic MUST explain that the marker enforces direct source-shape checks rather than transitive whole-program purity proof.

#### Scenario: Clean marked pure function stays clean

r[octet.purity_effect_lints.pure_attr_contract.allows_clean_marked_pure]

- GIVEN a marked function computes from explicit inputs, returns explicit outputs, and avoids the banned direct-effect checklist
- WHEN the lint runs
- THEN `pure_attr_contract` MUST NOT emit for that function.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.pure_attr_contract.catalog]

- GIVEN `pure_attr_contract` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, marker syntax docs, and downstream rollout guidance MUST include the lint identity and default level.
