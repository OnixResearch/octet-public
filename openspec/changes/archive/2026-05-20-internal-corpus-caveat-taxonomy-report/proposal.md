## Why

Octet object corpus receipts expose aggregate caveat counts, but operators cannot inspect a durable per-bucket taxonomy that explains which selected source buckets are blocked by dependency, effect, macro, or pure-cache caveats. That weakens reviewability before any HIR/MIR or downstream pilot work.

## What Changes

- Add an Octet object corpus caveat taxonomy report for selected source inputs.
- Include deterministic per-source-bucket caveat counts, pure-cache blocker counts, and boundary non-claims.
- Add a checked-in expanded internal corpus caveat taxonomy fixture and replay tests.

## Non-Goals

- No HIR/MIR extraction.
- No pure-cache eligibility promotion.
- No correctness, semantic-equivalence, or exhaustive Rust coverage claim.

## Impact

- Files: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, object-corpus fixtures.
- APIs: adds `cargo octet object corpus caveats --output-format human|json [PATHS...]`.
- Testing: focused `object_internal_corpus_caveat_taxonomy` tests plus full Octet verification.
