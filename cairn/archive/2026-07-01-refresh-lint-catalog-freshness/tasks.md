## Contract

- [x] [serial] Specify the inventory-derived catalog fields that README and lint-reference docs must reflect. r[octet.lint_catalog_freshness.docs_match_inventory]
- [x] [serial] Specify deterministic failure behavior for stale totals, stale domain counts, missing lint rows, and mismatched default levels. r[octet.lint_catalog_freshness.drift_detection]
- [x] [parallel] Identify any namespace-dependent catalog prose that must consume the namespace decision rather than choose it here. r[octet.lint_catalog_freshness.namespace_dependency]

## Implementation

- [x] [serial] Refresh README and `docs/lint-reference.md` totals, domain counts, default levels, and lint summaries from live inventory and diagnostic metadata. r[octet.lint_catalog_freshness.docs_match_inventory]
- [x] [serial] Add a pure docs/catalog freshness helper that accepts in-memory inventory/docs inputs and returns structured mismatches. r[octet.lint_catalog_freshness.drift_detection]
- [x] [parallel] Wire the freshness helper into the smallest existing test rail without requiring filesystem work inside the pure comparison core. r[octet.lint_catalog_freshness.drift_detection]

## Verification

- [x] [serial] Add positive tests proving current README and lint-reference catalog sections match live inventory-derived expectations. r[octet.lint_catalog_freshness.docs_match_inventory]
- [x] [serial] Add negative tests or fixtures proving an incorrect total, missing lint row, and mismatched default level fail with targeted diagnostics. r[octet.lint_catalog_freshness.drift_detection]
- [x] [serial] Run targeted docs/catalog tests, `nix develop -c cargo test --lib`, and Cairn validation/gates for this change. r[octet.lint_catalog_freshness.namespace_dependency]
