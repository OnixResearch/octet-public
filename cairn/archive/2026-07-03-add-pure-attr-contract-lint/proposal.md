## Why

Some functions need an explicit purity contract stronger than path-based core scopes. A source marker such as `#[tigerstyle::pure]` can let maintainers opt into strict checks for critical helpers and make violations visible at the function boundary.

## What Changes

- Define a `pure_attr_contract` lint for functions marked with Octet's recognized pure marker.
- Ban direct platform effects, ambient time/env/random, spawning, logging, global/interior mutation, mutating input capabilities, panics, and unchecked shell concerns inside marked functions.
- Require docs for marker syntax, namespace compatibility, suppression rules, and positive/negative fixtures.

## Impact

- **Files**: `src/purity/pure_attr_contract.rs`, attribute parsing helpers, registration, metadata, lint reference, templates, and UI fixtures.
- **Testing**: positive fixtures for marked pure violations and negative fixtures for clean marked functions, unmarked functions, and test-context behavior.
