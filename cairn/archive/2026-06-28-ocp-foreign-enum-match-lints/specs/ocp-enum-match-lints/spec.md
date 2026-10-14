## ADDED Requirements

### Requirement: foreign_enum_catch_all lint (OCP)

r[octet.ocp_enum_match.foreign_catch_all] Octet MUST ship a `foreign_enum_catch_all` safety lint that flags `_` match arms over enums defined in another crate, defaulting to `allow`.

#### Scenario: Foreign enum catch-all flagged

r[octet.ocp_enum_match.foreign_catch_all.flagged]

- GIVEN a `match` scrutinee resolves to an enum whose defining crate differs from the match's crate and a `_` arm is present
- WHEN the lint runs
- THEN it MUST emit an `allow`-level diagnostic naming the foreign enum and the catch-all arm.
- AND the remediation MUST suggest handling new variants explicitly or documenting why the catch-all is intentional.

#### Scenario: Own enum catch-all is not flagged by this lint

r[octet.ocp_enum_match.foreign_catch_all.own_enum]

- GIVEN a `match` over a same-crate enum uses a `_` arm
- WHEN the lint runs
- THEN it MUST emit zero diagnostics for `foreign_enum_catch_all`.
- AND macro-generated scrutinees MUST be skipped to avoid false positives.

### Requirement: fragile_exhaustive_enum_match lint (OCP)

r[octet.ocp_enum_match.fragile_exhaustive] Octet MUST ship a `fragile_exhaustive_enum_match` safety lint that flags exhaustive matches over same-crate enums without a `_` arm, defaulting to `allow`, and MUST honor a sealed-enum marker.

#### Scenario: Fragile exhaustive match flagged

r[octet.ocp_enum_match.fragile_exhaustive.flagged]

- GIVEN a `match` over a same-crate enum names all variants and has no `_` arm
- WHEN the lint runs
- THEN it MUST emit an `allow`-level diagnostic marking the match fragile to future variant additions.

#### Scenario: Sealed enum suppresses the diagnostic

r[octet.ocp_enum_match.fragile_exhaustive.sealed]

- GIVEN an enum is marked with the configured `sealed_enum_marker` (default `octet::sealed_enum`)
- WHEN the lint runs over an exhaustive match on that enum
- THEN it MUST emit zero diagnostics.
- AND the marker MUST be configurable through `dylint.toml`.

### Requirement: sealed_enum_marker config key

r[octet.ocp_enum_match.sealed_marker] The lint config MUST expose a `sealed_enum_marker` key that names the attribute used to mark deliberately-closed enums.

#### Scenario: Marker is configurable

r[octet.ocp_enum_match.sealed_marker.config]

- GIVEN a consumer sets `sealed_enum_marker = "my_crate::sealed"` in `dylint.toml`
- WHEN the `fragile_exhaustive_enum_match` lint resolves the marker
- THEN it MUST honor the configured attribute name instead of the default `octet::sealed_enum`.
