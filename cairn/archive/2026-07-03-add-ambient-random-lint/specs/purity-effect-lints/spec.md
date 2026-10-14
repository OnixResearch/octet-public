## ADDED Requirements

### Requirement: Ambient randomness lint

r[octet.purity_effect_lints.ambient_random] Octet MUST provide an `ambient_random` lint that flags direct ambient randomness or non-deterministic ID generation in production code and guides maintainers toward explicit seeds, injected generators, or shell-generated values.

#### Scenario: Ambient randomness is flagged

r[octet.purity_effect_lints.ambient_random.flags_random]

- GIVEN production Rust code directly calls a known thread-local RNG, OS randomness, `getrandom`, or UUID v4 generation API
- WHEN the lint runs
- THEN `ambient_random` MUST emit a diagnostic at the random source.
- AND the diagnostic MUST explain that randomness is hidden input for deterministic core logic.

#### Scenario: Injected randomness stays clean

r[octet.purity_effect_lints.ambient_random.allows_injected_randomness]

- GIVEN code receives an RNG trait object, seed value, nonce bytes, or generated identifier as an explicit input
- WHEN the lint runs
- THEN `ambient_random` MUST NOT emit for that source shape.

#### Scenario: Catalog and fixture coverage are fresh

r[octet.purity_effect_lints.ambient_random.catalog]

- GIVEN `ambient_random` is registered
- WHEN metadata and fixture freshness validation runs
- THEN README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance MUST include the lint identity and default level.
