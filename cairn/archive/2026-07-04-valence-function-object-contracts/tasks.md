## Phase 1: Implementation

- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_normalization] Move Function Object v1 normalization ownership into `valence-core` or introduce a typed normalized-source contract carrying the normalization algorithm.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_hash_payload] Replace inline hash-material JSON construction with an explicit typed hash payload helper.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_typed_api] Replace stringly internal Function Object errors/statuses/blockers with typed Rust variants that serialize to existing stable strings.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_shell] Keep `cargo-octet` responsible only for scanner/filesystem/orchestration shell behavior around Function Object construction.

## Phase 2: Verification

- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_normalization.core_owned] Verify the positive path constructs identical Function Object v1 records when given the same source facts and declared normalization algorithm.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_normalization.rejects_mismatch] Verify negative mismatched or unsupported normalization inputs fail closed with a typed error and no object hash.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_hash_payload.reviewable] Verify hash-payload tests enumerate included/excluded fields and preserve existing object hashes unless an explicit migration fixture is part of the change.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_typed_api.serializes_compatibly] Verify typed errors/statuses/blockers serialize to the existing Function Object v1 strings and reject malformed construction inputs.
- [x] [serial] r[octet.valence_project_extraction.phase4.function_object_gates.compatibility] Run focused `valence-core` Function Object tests, focused `cargo-octet` object tests, object corpus gates, `valence-object-evidence`, and Cairn validation/gates.
- [x] [serial] Run Cairn validate plus proposal/design/tasks gates for this change package before implementation closeout. [depends:cairn-gates]

## Notes

- The hardened core introduces a typed `NormalizedFunctionSource` (algorithm-tagged, fails closed on mismatch), a typed `ConstructionError`, typed `DependencyIdentityStatus`/`EffectCaveatKind`/`EffectCaveatStatus`/`MacroCaveatStatus`/`PureCacheBlocker` enums, and a reviewable `FunctionObjectHashPayload`. Serialized Function Object v1 DTO fields remain `&'static str` populated via `as_str()`, so the serialized v1 contract is byte-identical.
- Object hash byte-stability is proven by the added `legacy_inline_json_payload_matches_new_canonical_json` test, which reconstructs the prior inline `serde_json::json!` payload and asserts byte-identical canonical JSON, and by `object_corpus_compatibility_positive_v1_fixture_replays_cleanly` replaying an old-code-generated 2-object fixture cleanly under the hardened core.
- The checked-in `internal-corpus-receipt.json`, `expanded-internal-corpus-receipt.json`, and `expanded-internal-corpus-caveat-taxonomy.json` fixtures were refreshed because they were stale relative to 2026-07-04 source additions (`d8f8655`, `af331b0`, `4e90163`, `4944dca`, `b988c78`, `24d0dd1`) that added functions to corpus-scanned `cargo-octet/src/*.rs` files without refreshing the receipts. This is a source-growth refresh, not a Function Object v1 hash-material migration: per-object hashes for pre-existing objects are unchanged, proven by the byte-identical payload test and the old-code compatibility fixture replay. The `expanded-internal-corpus-taxonomy.json` path-list fixture is not byte-compared and was left as-is; it remains a pre-existing non-fresh item if its source selection drifts.
