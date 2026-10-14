## Phase 1: Implementation

- [x] I1 Add or narrow focused MIR inventory/roundtrip test coverage so the test proves selected MIR shapes `checked-arithmetic` and `call-return`, inventory replay commands, unsupported selector rejection, and default `unsupported-planned` boundaries.
- [x] I2 Add named Nix check `mir-supplement-inventory-gate` and compose it into `valence-object-evidence` while preserving the existing MIR fixture PASS line for compatibility.
- [x] I3 Update contributor/current evidence docs to name selected MIR inventory coverage and non-proof boundaries without broad MIR extraction claims.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "MIR fixture inventory lists selected shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs and cargo-octet/tests/fixtures/mir-spike/*`; expect=`test result: ok and output/assertions cover exactly checked-arithmetic and call-return`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/mir-fixture-inventory-tests.txt`
- [x] V2 Verify scenario "MIR fixture inventory commands roundtrip selected fixture evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`inventory-emitted MIR emit/check command expectations`; expect=`checked-arithmetic and call-return emit pilot-supported evidence and replay successfully while unsupported shapes fail closed`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/mir-fixture-inventory-tests.txt`
- [x] V3 Verify scenario "Aggregate gate composes object evidence rails" with command=`nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; input=`flake.nix aggregate Valence object evidence check`; expect=`aggregate build passes and all composed rail PASS lines are printed`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/valence-object-evidence.txt`
- [x] V4 Verify scenario "Aggregate gate composes MIR inventory rail" with command=`nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L && nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; input=`focused MIR inventory check plus aggregate output`; expect=`mir-supplement-inventory-gate passes and aggregate prints valence_object_evidence.mir_supplement_inventory_gate: PASS plus existing mir_supplement_fixture_gate: PASS`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/mir-inventory-nix-gate.txt`
- [x] V5 Verify scenario "Contributor docs name the aggregate command and MIR boundaries" with command=`python3 openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/check-doc-boundaries.py`; input=`docs/contributor-guide.md docs/current-head-valence-evidence.md`; expect=`PASS for aggregate command, selected MIR inventory wording, and non-proof boundaries`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/doc-boundaries.txt`
- [x] V6 Verify final OpenSpec and diff hygiene with command=`openspec validate --all --strict --json && git diff --check`; input=`active change plus repository diff`; expect=`all specs/changes pass and diff check emits no whitespace errors`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/final-validation.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review docs/current-head-valence-evidence.md docs/contributor-guide.md flake.nix`; input=`final aggregate output and focused MIR inventory evidence`; asserts=`MIR support is described as selected-fixture-only and no broader MIR extraction/correctness claim was introduced`; evidence=`openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/human-boundary-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: MIR fixture inventory lists selected shapes` -> `V1` -> `openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/mir-fixture-inventory-tests.txt`
- `Scenario: MIR fixture inventory commands roundtrip selected fixture evidence` -> `V2` -> `openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/mir-fixture-inventory-tests.txt`
- `Scenario: Aggregate gate composes object evidence rails` -> `V3` -> `openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/valence-object-evidence.txt`
- `Scenario: Aggregate gate composes MIR inventory rail` -> `V4` -> `openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/mir-inventory-nix-gate.txt`
- `Scenario: Contributor docs name the aggregate command and MIR boundaries` -> `V5` -> `openspec/changes/archive/2026-05-22-add-mir-inventory-aggregate-rail/evidence/doc-boundaries.txt`
