## Task ID rules

- `I<n>` — implementation-only.
- `V<n>` — deterministic verification with command/input/expect/evidence.
- `H<n>` — human/oracle checkpoint with command=, input=, asserts=, evidence=.

## Phase 1: Implementation

- [x] I1 Add named object-corpus fixture freshness Nix check.
- [x] I2 Compose the freshness check into aggregate Valence object evidence with PASS output.
- [x] I3 Document freshness rail obligations in contributor and task-template guidance.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Named fixture freshness rail passes" with command=`nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L`; input=`cargo-octet/tests/fixtures/object-corpus/{internal-corpus-receipt.json,expanded-internal-corpus-receipt.json,expanded-internal-corpus-caveat-taxonomy.json}`; expect=`3 passed; 0 failed`; evidence=`openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/object-corpus-fixture-freshness.txt`
- [x] V2 Verify scenario "Aggregate Valence gate names freshness rail" with command=`nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; input=`flake.nix`; expect=`valence_object_evidence.object_corpus_fixture_freshness_gate: PASS`; evidence=`openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/valence-object-evidence.txt`
- [x] V3 Verify scenario "Guidance names freshness obligation" with command=`python3 openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/check-guidance.py`; input=`docs/contributor-guide.md and openspec/templates/tasks.md`; expect=`guidance: PASS`; evidence=`openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/guidance-check.txt`
- [x] V4 Verify requirement "Object corpus fixture freshness rail" with command=`openspec validate --all --strict --json && git diff --check`; input=`repo`; expect=`14 specs passed, 0 failed and no whitespace errors`; evidence=`openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/final-validation.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`manual review of flake/docs/task-template diff`; input=`flake.nix, docs/contributor-guide.md, openspec/templates/tasks.md`; asserts=`fixture freshness is named and composed without broad correctness or extraction claims`; evidence=`openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/human-boundary-review.txt`

## Change Status
- Status: done
- Open tasks: none

## Verification Coverage

- `Scenario: Named fixture freshness rail passes` -> `V1` -> `openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/object-corpus-fixture-freshness.txt`
- `Scenario: Aggregate Valence gate names freshness rail` -> `V2` -> `openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/valence-object-evidence.txt`
- `Scenario: Guidance names freshness obligation` -> `V3` -> `openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/guidance-check.txt`
- `Requirement: Object corpus fixture freshness rail` -> `V4` -> `openspec/changes/archive/2026-05-22-add-object-corpus-fixture-freshness-rail/evidence/final-validation.txt`
