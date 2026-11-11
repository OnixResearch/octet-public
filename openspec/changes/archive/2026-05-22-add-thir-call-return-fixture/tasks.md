## Phase 0: Stage gates before implementation

- [x] [serial] H1 Human/oracle checkpoint ✅ evidence captured (proposal gate PASS): command=`openspec_gate stage=proposal change=add-thir-call-return-fixture`; input=`proposal.md and standards-codification delta`; asserts=`proposal scope is closed to selected THIR fixture expansion and does not claim general extraction`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/proposal-gate.txt`
- [x] [depends:H1] H2 Human/oracle checkpoint ✅ evidence captured (design gate PASS): command=`openspec_gate stage=design change=add-thir-call-return-fixture`; input=`design.md plus delta spec`; asserts=`design keeps parser/inventory/runtime/docs/test surfaces synchronized and fail-closed`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/design-gate.txt`
- [x] [depends:H2] H3 Human/oracle checkpoint ✅ evidence captured (tasks gate PASS): command=`openspec_gate stage=tasks change=add-thir-call-return-fixture`; input=`tasks.md plus delta spec`; asserts=`each scenario maps to an exact verification task and evidence path before implementation`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/tasks-gate.txt`

## Phase 1: Implementation

- [x] [depends:H3] I1 Add the `call-return` THIR fixture source and closed selector mapping.
- [x] [depends:I1] I2 Update THIR fixture runtime metadata, contract metadata, docs, and parser/help/test expectations for selected shapes `checked-arithmetic` and `call-return` while keeping default THIR `unsupported-planned`.

## Phase 2: Deterministic verification

- [x] [depends:I2] V1 Verify scenario "THIR fixture inventory lists selected shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet thir_fixture -- --nocapture'`; input=`cargo-octet/tests/fixtures/thir-spike/checked.rs and cargo-octet/tests/fixtures/thir-spike/call_return.rs`; expect=`PASS with inventory shapes exactly checked-arithmetic and call-return`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/thir-fixture-tests.txt`
- [x] [depends:I2] V2 Verify scenario "THIR call-return fixture emits pilot-supported evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet thir_fixture -- --nocapture'`; input=`--family thir --spike rustc-thir-fixture --fixture-shape call-return`; expect=`PASS with pilot-supported call-return emit/check and mismatch rejection`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/thir-fixture-tests.txt`
- [x] [depends:I2] V3 Verify scenario "THIR rejects unsupported fixture shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'`; input=`--family thir --spike rustc-thir-fixture --fixture-shape branch-control-flow and --family thir --spike rustc-thir-fixture without --fixture-shape`; expect=`PASS deterministic unsupported-shape and missing-explicit-shape diagnostics`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/parser-tests.txt`
- [x] [depends:I2] V4 Verify scenario "THIR default contract remains unsupported-planned after fixture expansion" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet supplement -- --nocapture && cargo test -p cargo-octet object_corpus_contract -- --nocapture'`; input=`object supplement docs/contract surfaces`; expect=`PASS with THIR contract metadata status unsupported-planned and selected_fixture_shapes checked-arithmetic,call-return`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/supplement-regression-tests.txt`
- [x] [depends:I2] V5 Verify active OpenSpec change with command=`openspec validate add-thir-call-return-fixture --strict --json`; input=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture`; expect=`success`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/openspec-validate.txt`

## Phase 3: Closeout verification

- [x] [depends:V1,V2,V3,V4,V5] A1 Sync and archive the completed OpenSpec change with command=`openspec archive add-thir-call-return-fixture --yes`; input=`completed active change`; expect=`archive under openspec/changes/archive/<date>-add-thir-call-return-fixture and synced canonical spec`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/archive.txt`
- [x] [depends:A1] V6 Verify all specs after archive sync with command=`openspec validate --all --strict --json`; input=`openspec/`; expect=`success`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/final-openspec-all.txt`
- [x] [depends:V5] V7 Verify whitespace hygiene with command=`git diff --check`; input=`tracked change diff`; expect=`no whitespace errors`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/git-diff-check.txt`
- [x] [depends:A1,V1,V2,V3,V4,V5,V6,V7] H4 Done-review/archive closeout: command=`done-review after archive`; input=`implementation diff, archived change, synced canonical spec, verification evidence`; asserts=`no blocking overclaim/spec/archive/task drift`; evidence=`openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/done-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: THIR fixture inventory lists selected shapes` -> `V1` -> `openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/thir-fixture-tests.txt`
- `Scenario: THIR call-return fixture emits pilot-supported evidence` -> `V2` -> `openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/thir-fixture-tests.txt`
- `Scenario: THIR rejects unsupported fixture shapes` -> `V3` -> `openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/parser-tests.txt`
- `Scenario: THIR default contract remains unsupported-planned after fixture expansion` -> `V4` -> `openspec/changes/archive/2026-05-22-add-thir-call-return-fixture/evidence/supplement-regression-tests.txt`
