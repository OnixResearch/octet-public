## Why

`docs/rust-idiom-guidance.md` still describes `borrowed_argument_types` as a future lint candidate even though Octet now ships it as a warning lint. This is a small instance of a broader drift class: guidance pages can become stale even when catalog freshness keeps README and lint-reference tables correct.

Consumers use guidance pages to decide which patterns are advisory, checked, or future work, so stale enforcement-tier wording should fail closed.

## What Changes

- Refresh the stale Rust idiom guidance entry for `borrowed_argument_types`.
- Add a guidance-freshness contract for docs that name lint enforcement tiers outside the main catalog.
- Add deterministic checks for known guidance rows that reference shipped lints.

## Impact

- **Files**: `docs/rust-idiom-guidance.md`, documentation freshness tests or fixtures, and possibly lint metadata if guidance rows are derived from catalog metadata later.
- **Testing**: docs freshness positive test for current guidance, negative fixture for stale future-lint wording, and Cairn validation/gates.
