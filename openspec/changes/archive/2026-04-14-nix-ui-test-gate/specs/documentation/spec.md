## ADDED Requirements

### Requirement: Nix Check Path Exercises UI Lint Fixtures

The project MUST execute real Dylint UI fixtures as part of its nix-backed
validation path.

The nix check path MUST provide whatever rustup compatibility wiring the UI
harness needs, rather than silently skipping when `rustup` is absent.

The repository MUST keep at least one checked-in UI fixture whose expected
stderr is validated during that nix check path.

#### Scenario: Running nix flake check

- GIVEN a maintainer runs `nix flake check`
- WHEN the test suite reaches the UI harness
- THEN it runs checked-in Dylint UI fixtures instead of returning early because
  `rustup` is missing

#### Scenario: UI regression in nix path

- GIVEN a checked-in UI fixture's lint output changes unexpectedly
- WHEN the nix validation path runs
- THEN the UI test fails and reports the stderr mismatch
