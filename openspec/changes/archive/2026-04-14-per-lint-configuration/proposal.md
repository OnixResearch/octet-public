## Why

Consumer `dylint.toml` can tune thresholds (`function_length_max`,
`nesting_max_depth`, etc.) but cannot enable or disable individual lints.
The only options today are source-level `#[allow(tigerstyle::...)]`
attributes or not running tigerstyle at all. This forces consumers into
an all-or-nothing choice per Dylint run: either all 26 lints fire, or
they scatter `#[allow]` attributes everywhere.

The Aspen rollout playbook works around this by documenting rollout
profiles such as smoke, boundary-first, and inventory-only, but the
actual enforcement is still social, not mechanical.

## What Changes

- Add a `disabled_lints` list to the `[tigerstyle]` section of `dylint.toml`
  that prevents named lints from firing
- Add a `lint_levels` table that lets consumers set per-lint severity
  (`allow`, `warn`, `deny`)
- The config is consumer-owned, not baked into tigerstyle defaults

Example `dylint.toml`:

```toml
[tigerstyle]
function_length_max = 100

# Phase 1: boundary-first profile
disabled_lints = [
    "ignored_result",
    "no_unwrap",
    "no_panic",
    "unchecked_narrowing",
    "unbounded_loop",
]

# Phase 2: promote ignored_result to deny
[tigerstyle.lint_levels]
ignored_result = "deny"
compound_assertion = "deny"
```

- Unknown lint names or invalid level values warn and are ignored
- Update `templates/dylint.toml` with commented examples for each control

## Capabilities

### New Capabilities
- `per-lint-disable`: Consumers can disable individual lints in `dylint.toml`
- `per-lint-severity`: Consumers can set allow/warn/deny per lint

### Modified Capabilities
- `config`: Extended config schema with new fields

## Impact

- **Files**: `src/config.rs`, `src/lib.rs` (registration conditional on
  config), `templates/dylint.toml`
- **APIs**: New fields on `Config` struct
- **Dependencies**: None
- **Testing**: Config parsing and warning-path tests, integration tests for
  disabled lints, allow/warn/deny overrides, fixture-local config cases, and
  a human Aspen replay when downstream rollout docs/templates change so the
  primary consumer proof stays aligned with the shipped guidance
