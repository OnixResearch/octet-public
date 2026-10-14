## Context

The existing `object_internal_corpus` test filter includes fixture freshness checks for internal receipts, expanded receipts, and caveat taxonomy. After THIR `branch-control-flow`, a follow-up current-head sweep found stale receipt hashes; a named sub-rail would have made the missing refresh obligation explicit.

## Decision

Add `objectCorpusFixtureFreshnessGate` using the existing `mkFingerprintDocsCheck` helper with test filter `matches_fresh`. This selects the three current tests whose names assert fresh fixture parity:

- `object_internal_corpus_gate_fixture_matches_fresh_receipt`
- `object_internal_corpus_expanded_fixture_matches_fresh_receipt_and_preserves_boundaries`
- `object_internal_corpus_expanded_caveat_taxonomy_fixture_matches_fresh_report`

Expose it as `checks.x86_64-linux.object-corpus-fixture-freshness` and require it in `valenceObjectEvidenceGate` before printing `valence_object_evidence.object_corpus_fixture_freshness_gate: PASS`.

## Alternatives

- Rely only on `object-corpus-gate`: rejected because it is broader and the fixture-refresh failure is less operator-visible.
- Add new duplicate Rust tests: rejected because existing focused tests already express the invariant; this change names and composes them.

## Risks / Boundaries

This is a bounded local fixture freshness rail. It does not prove correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, or cross-toolchain determinism.
