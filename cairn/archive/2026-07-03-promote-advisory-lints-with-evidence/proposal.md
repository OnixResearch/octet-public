## Why

Several useful Octet lints are allow-by-default because their initial false-positive risk was unknown or consumer impact needed evidence. Promotion should be possible, but only after dogfood/downstream runs show low noise, clear remediation, and stable suppression behavior.

Without an explicit promotion contract, default-level changes can become subjective and surprise consumers.

## What Changes

- Define evidence gates for promoting allow-by-default lints to `warn` or starter profiles.
- Apply the contract first to candidates such as `raw_arithmetic_overflow`, `ambiguous_params`, `multi_lock_ordering`, `defaulting_trait_method`, and `numeric_units`.
- Require migration notes, fixture coverage, dogfood counts, and downstream profile changes before any default-level change.

## Impact

- **Files**: `crates/octet-lint-metadata`, lint declarations, docs, templates, downstream rollout docs, UI fixtures, and saved evidence artifacts.
- **Testing**: default-level snapshot tests, catalog freshness tests, targeted dogfood runs, and negative compatibility tests for source attrs and config overrides.
