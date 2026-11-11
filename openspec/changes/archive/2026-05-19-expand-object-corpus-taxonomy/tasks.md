## Phase 1: Implementation

- [x] I1 Add expanded internal object corpus taxonomy and receipt fixtures.
- [x] I2 Add expanded corpus replay, freshness, taxonomy, and boundary tests.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Expanded taxonomy names selected source buckets" with focused expanded-corpus tests; save output at `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`.
- [x] V2 Verify scenario "Expanded corpus fixture replays cleanly" with focused expanded-corpus tests; save output at `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`.
- [x] V3 Verify scenario "Expanded corpus preserves bounded evidence claims" with focused expanded-corpus tests; save output at `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`.
- [x] V4 Run full Octet verification; save output at `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/final-verification.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`nix develop -c bash -c 'cargo test -p cargo-octet object_internal_corpus_expanded -- --nocapture'`; input=`cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-taxonomy.json`; asserts=`expanded corpus selection stays internal to Octet/Slotcar and preserves non-proof boundaries`; evidence=`openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Expanded taxonomy names selected source buckets` -> `V1` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`
- `Scenario: Expanded corpus fixture replays cleanly` -> `V2` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`
- `Scenario: Expanded corpus preserves bounded evidence claims` -> `V3` -> `openspec/changes/archive/2026-05-19-expand-object-corpus-taxonomy/evidence/expanded-object-corpus-tests.txt`
