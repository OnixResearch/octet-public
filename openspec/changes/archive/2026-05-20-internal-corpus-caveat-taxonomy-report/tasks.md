## Phase 1: Implementation

- [x] I1 Add the object corpus caveat taxonomy report DTO and CLI command.
- [x] I2 Add the expanded internal caveat taxonomy fixture and integration tests.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Expanded internal taxonomy fixture is deterministic" with focused test output saved at `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-tests.txt`.
- [x] V2 Verify scenario "Caveat taxonomy names per-source blockers" with focused test output saved at `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-tests.txt`.
- [x] V3 Verify scenario "Caveat taxonomy preserves non-proof boundaries" with focused test output saved at `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-tests.txt`.
- [x] V4 Verify full Octet gate with output saved at `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/final-verification.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`python3 - <<'PY' ... inspect caveat taxonomy fixture ... PY`; input=`cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-caveat-taxonomy.json`; asserts=`fixture contains sorted source buckets and false boundary claims`; evidence=`openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-fixture.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Expanded internal taxonomy fixture is deterministic` -> `V1` -> `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-tests.txt`
- `Scenario: Caveat taxonomy names per-source blockers` -> `V2` -> `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-tests.txt`
- `Scenario: Caveat taxonomy preserves non-proof boundaries` -> `V3` -> `openspec/changes/archive/2026-05-20-internal-corpus-caveat-taxonomy-report/evidence/caveat-taxonomy-tests.txt`
