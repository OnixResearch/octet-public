## ADDED Requirements

### Requirement: Content Identity Lint Cache [r[content-identity-lint-cache]]

The system MUST support an optional deterministic lint-result cache keyed by Slotcar version, rustc/toolchain identity, lint config hash, crate graph hash, lint ID, and semantic item identity. The cache MUST be disabled or bypassed for findings that cannot prove deterministic inputs.

#### Scenario: Unchanged item uses cached finding [r[content-identity-lint-cache.scenario.unchanged-item-uses-cached-finding]]

- GIVEN an item identity, config hash, crate graph hash, and toolchain identity are unchanged between runs
- WHEN `cargo slotcar check --cache` runs again
- THEN deterministic findings for that item are loaded from cache and reported with cache-hit metadata

#### Scenario: Config change invalidates cache [r[content-identity-lint-cache.scenario.config-change-invalidates-cache]]

- GIVEN the lint config hash changes between two runs
- WHEN `cargo slotcar check --cache` runs
- THEN previous cached findings are not reused for affected lint IDs

#### Scenario: Non-deterministic input bypasses cache [r[content-identity-lint-cache.scenario.non-deterministic-input-bypasses-cache]]

- GIVEN an item lacks semantic identity or depends on unsupported macro expansion context
- WHEN the cache key is computed
- THEN the item is marked cache-ineligible and linting runs normally

