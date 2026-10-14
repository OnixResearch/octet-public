## ADDED Requirements

### Requirement: Missing const fn lint

r[octet.purity_effect_lints.missing_const_fn] Octet SHOULD provide a `missing_const_fn` lint that identifies conservative, const-compatible helper functions that are not declared `const fn`.

#### Scenario: Const-eligible helper is flagged

r[octet.purity_effect_lints.missing_const_fn.flags_const_eligible]

- GIVEN a Rust helper function is not declared `const`
- WHEN its signature and body match Octet's conservative const-eligible subset
- THEN `missing_const_fn` SHOULD emit an advisory diagnostic at the function signature.
- AND any suggestion MUST use an applicability level that reflects whether compile-proof or autofix proof exists.

#### Scenario: Non-const or intentionally flexible code stays clean

r[octet.purity_effect_lints.missing_const_fn.allows_non_const]

- GIVEN a function is async, allocates, calls non-const functions, performs shell effects, implements a trait method, is already `const`, or falls under a public-API guardrail
- WHEN the lint runs
- THEN `missing_const_fn` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.missing_const_fn.catalog]

- GIVEN `missing_const_fn` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
