## Phase 1: Implementation

- [x] I1 Add selected MIR `call-return` fixture source and parser/runtime registry mapping.
- [x] I2 Update MIR supplement contract metadata, docs, and tests for exact selected selectors.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "MIR fixture inventory lists selected shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`cargo-octet/tests/fixtures/mir-spike/{checked.rs,call_return.rs}`; expect=`PASS inventory lists exactly checked-arithmetic and call-return`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/mir-fixture-tests.txt`
- [x] V2 Verify scenario "MIR call-return fixture emits pilot-supported evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`--family mir --spike rustc-mir-fixture --fixture-shape call-return`; expect=`PASS schema octet.function-object-mir-supplement.v1 and status pilot-supported`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/mir-fixture-tests.txt`
- [x] V3 Verify scenario "MIR rejects unsupported fixture shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'`; input=`--fixture-shape checked and --fixture-shape branch-control-flow`; expect=`PASS deterministic usage errors and no alias acceptance`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/parser-tests.txt`
- [x] V4 Verify scenario "MIR default contract remains unsupported-planned after fixture expansion" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_corpus_contract -- --nocapture'`; input=`cargo octet object corpus contract`; expect=`PASS MIR family status unsupported-planned with selected_fixture_shapes checked-arithmetic,call-return`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/object-corpus-contract-tests.txt`
- [x] V5 Verify aggregate MIR fixture gate with command=`nix build .#checks.x86_64-linux.mir-supplement-fixture-gate --no-link -L`; input=`flake check mir-supplement-fixture-gate`; expect=`PASS named MIR supplement fixture gate`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/mir-supplement-fixture-gate.txt`
- [x] V6 Verify aggregate Valence evidence gate with command=`nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; input=`flake check valence-object-evidence`; expect=`PASS aggregate Valence object evidence gate`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/valence-object-evidence.txt`
- [x] V7 Verify OpenSpec and whitespace with command=`openspec validate --all --strict --json && git diff --check`; input=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture and working tree diff`; expect=`PASS strict validation and diff check`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/final-validation.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review_gate_evidence + reviewer prompt`; input=`working tree after implementation and focused verification`; asserts=`review warnings were bookkeeping-only and fixed before closeout`; evidence=`openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/done-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: MIR fixture inventory lists selected shapes` -> `V1` -> `openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/mir-fixture-tests.txt`
- `Scenario: MIR call-return fixture emits pilot-supported evidence` -> `V2` -> `openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/mir-fixture-tests.txt`
- `Scenario: MIR rejects unsupported fixture shapes` -> `V3` -> `openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/parser-tests.txt`
- `Scenario: MIR default contract remains unsupported-planned after fixture expansion` -> `V4` -> `openspec/changes/archive/2026-05-22-add-mir-call-return-fixture/evidence/object-corpus-contract-tests.txt`
