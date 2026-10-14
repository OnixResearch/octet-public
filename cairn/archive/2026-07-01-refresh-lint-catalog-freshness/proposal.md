## Why

The audit found a concrete consumer-facing drift: `README.md` and `docs/lint-reference.md` say Octet ships 44 lints, while the checked lint inventory contains 53 entries. The domain counts in the docs are stale too. Because consumers use those docs to decide rollout scope, stale counts and tables make the tool look less reliable than the actual test coverage.

Octet needs a focused catalog-freshness change that refreshes the current docs from the live inventory and adds a small fail-closed test so the same drift cannot recur while the broader metadata centralization work is still pending.

## What Changes

- Refresh README and lint-reference catalog counts, domain counts, default levels, and summaries from the live lint inventory and diagnostic metadata.
- Add deterministic freshness validation that compares consumer-facing catalog prose/tables against the registered lint inventory and diagnostic metadata.
- Keep this change scoped to freshness and documentation correctness; broader metadata deduplication remains in `centralize-lint-metadata`.

## Impact

- **Files**: `README.md`, `docs/lint-reference.md`, `src/lib.rs` tests or a focused docs-positioning/catalog test, and related fixtures under `tests/fixtures/` if a generated snapshot is introduced.
- **Testing**: positive freshness test for current docs, negative drift fixture/test proving stale counts fail, plus targeted library/docs tests.
