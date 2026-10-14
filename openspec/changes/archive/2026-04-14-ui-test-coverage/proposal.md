## Why

Only 2 of 25 lints (`ambient_clock`, `nested_conditionals`) have UI test
fixtures with checked-in `.stderr` expectations. The nix `checks.ui` gate
validates those two but the remaining 23 lints have no regression protection
against diagnostic wording changes, span drift, or false-positive
regressions. Any nightly rustc update can silently break lint output for
lints that lack fixtures.

## What Changes

- Add UI test fixtures (`.rs` + `.stderr`) for all 25 lints under `ui_tests/`
- Each fixture exercises at least the primary fire case and one
  should-not-fire case from the lint's spec scenario table
- The nix `checks.ui` gate runs all fixtures, not just the current two
- Fixtures for configurable lints (`function_length`, `assertion_density`,
  `nesting_max_depth`, `acronym_allowlist`) test both default and
  overridden thresholds

## Capabilities

### New Capabilities
- `ui-regression-gate`: Every lint has at least one UI fixture validated
  by `nix flake check`

### Modified Capabilities
- `checks.ui`: Expands from 2 fixtures to 25 base lint fixtures plus configurable override suites

## Impact

- **Files**: `ui_tests/*.rs`, `ui_tests/*.stderr`, possibly
  `tests/support/ui-driver/` if the harness needs adjustment
- **APIs**: None
- **Dependencies**: None
- **Testing**: The UI fixtures ARE the testing — each fixture is a
  self-contained regression test
