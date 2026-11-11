## ADDED Requirements

### Requirement: Per-Lint Configuration Support

The project MUST support disabling individual lints and overriding lint
severity through the consumer-owned `dylint.toml`.

The consumer-owned `[tigerstyle]` section in `dylint.toml` MUST support these
optional controls and behaviors:

- an optional `disabled_lints` list that prevents named lints from registering
- an optional `lint_levels` table that maps lint names to `allow`, `warn`, or
  `deny`
- omitted `disabled_lints` and `lint_levels` keys default to empty collections
  and preserve the existing default lint levels
- a warning (not error) for unknown lint names in config
- a warning (not error) for invalid `lint_levels` values, with the invalid
  override ignored
- a warning (not error) when a lint appears in both `disabled_lints` and
  `lint_levels`, with the level override ignored

#### Scenario: Omitted per-lint controls keep existing behavior

- GIVEN a `dylint.toml` without `disabled_lints` or `lint_levels`
- WHEN the lint suite loads config
- THEN both controls default to empty and lints keep their default levels

#### Scenario: Disabling a noisy lint

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- WHEN the lint suite runs
- THEN `no_unwrap` produces no diagnostics

#### Scenario: Unknown lint name warns and continues

- GIVEN `disabled_lints = ["not_a_real_lint"]` in `dylint.toml`
- WHEN the lint suite loads config
- THEN it emits a warning and continues with the remaining config

#### Scenario: Invalid lint level warns and keeps default

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "fatal"` in `dylint.toml`
- WHEN the lint suite loads config
- THEN it emits a warning and `no_unwrap` keeps its default level

#### Scenario: Disabled lint beats level override

- GIVEN `disabled_lints = ["no_unwrap"]` and `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN the lint suite loads config and runs
- THEN it emits a warning about the conflicting config and `no_unwrap` stays disabled

#### Scenario: Allow level suppresses diagnostics

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "allow"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code
- THEN it emits no diagnostic

#### Scenario: Warn level keeps warning behavior

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "warn"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code
- THEN it emits a warning

#### Scenario: Deny level upgrades to error

- GIVEN `[tigerstyle.lint_levels] no_unwrap = "deny"` in `dylint.toml`
- WHEN `no_unwrap` sees violating code
- THEN it emits an error instead of a warning

#### Scenario: Disabled lint beats source-level attrs

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- AND source code requests a source-level lint attribute for `tigerstyle::no_unwrap`
- WHEN the lint suite runs on code that would otherwise violate `no_unwrap`
- THEN `no_unwrap` still emits no diagnostic

