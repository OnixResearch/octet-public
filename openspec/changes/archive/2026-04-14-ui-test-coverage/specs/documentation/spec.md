## MODIFIED Requirements

### Requirement: Nix Check Path Exercises UI Lint Fixtures

The project MUST execute real Dylint UI fixtures as part of its nix-backed
validation path.

The nix check path MUST provide whatever rustup compatibility wiring the UI
harness needs, rather than silently skipping when `rustup` is absent.

The repository MUST keep a checked-in UI fixture whose expected stderr is
validated during that nix check path for every shipped tigerstyle lint.

The repository MUST keep checked-in config-override UI fixtures for every
configurable tigerstyle behavior:

- `function_length_max`
- `nesting_max_depth`
- `assertion_density_min_lines` / `assertion_density_min_assertions`
- `acronym_allowlist`

The UI harness MUST assert the expected compiler exit status for each fixture,
so deny-level fixtures still prove the command failed for the right reason
instead of passing on stderr text alone.

#### Scenario: Running nix flake check

- GIVEN a maintainer runs `nix flake check`
- WHEN the test suite reaches the UI harness
- THEN it runs checked-in Dylint UI fixtures instead of returning early because
  `rustup` is missing

#### Scenario: Lint-wide regression coverage

- GIVEN a shipped tigerstyle lint changes its diagnostic wording or stops firing
- WHEN the nix validation path runs
- THEN the matching checked-in fixture fails and reports the stderr mismatch

#### Scenario: Config-override regression coverage

- GIVEN a maintainer changes config parsing or a configurable threshold
- WHEN the nix validation path runs
- THEN the checked-in override fixtures catch regressions in the configured
  behavior

#### Scenario: Deny-level fixture exits non-zero

- GIVEN a fixture that intentionally triggers a deny-by-default lint such as
  `verified_purity`
- WHEN the UI harness runs it
- THEN the harness checks that the compiler command failed and that stderr
  matches the expected deny-level diagnostic
