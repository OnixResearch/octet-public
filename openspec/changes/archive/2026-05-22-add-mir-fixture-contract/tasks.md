## Phase 0: Stage gates

- [x] [serial] H1 Human/oracle checkpoint: command=`pi --no-session -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call openspec_gate for stage=proposal change=add-mir-fixture-contract and print the full result."`; input=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/proposal.md and specs/standards-codification/spec.md`; asserts=`proposal gate accepts selected-MIR-fixture scope without general MIR extractor overclaim`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/proposal-gate.txt`
- [x] [depends:H1] H2 Human/oracle checkpoint: command=`pi --no-session -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call openspec_gate for stage=design change=add-mir-fixture-contract and print the full result."`; input=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/design.md`; asserts=`design gate accepts parser/inventory/runtime/docs/test synchronization plan`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/design-gate.txt`
- [x] [depends:H2] H3 Human/oracle checkpoint: command=`pi --no-session -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call openspec_gate for stage=tasks change=add-mir-fixture-contract and print the full result."`; input=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/tasks.md`; asserts=`tasks gate accepts one verification coverage row per MIR scenario`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/tasks-gate.txt`

## Phase 1: Implementation

- [x] [depends:H3] I1 Add MIR fixture CLI scope and closed selector parsing.
- [x] [depends:I1] I2 Add MIR fixture inventory/runtime supplement DTO support.
- [x] [depends:I2] I3 Add checked MIR fixture source and focused runtime/parser tests.
- [x] [depends:I3] I4 Update operator docs and aggregate Valence evidence gate wiring.

## Phase 2: Deterministic verification

- [x] [depends:I4] V1 Verify scenario "MIR fixture inventory lists selected shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs`; expect=`PASS inventory asserts schema octet.mir-fixture-inventory.v1, family mir, scope rustc-mir-fixture-spike, default_shape checked-arithmetic, exactly one shape, fixture path cargo-octet/tests/fixtures/mir-spike/checked.rs, function checked, support status pilot-supported, and selected-fixture-only boundary`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/mir-fixture-tests.txt`
- [x] [depends:I4] V2 Verify scenario "MIR checked-arithmetic fixture emits pilot-supported evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`cargo-octet/tests/fixtures/mir-spike/checked.rs`; expect=`PASS emit/check asserts schema octet.function-object-mir-supplement.v1, status pilot-supported, object hash linkage, fixture_context.detail path/function/shape/object hash, mir_fixture_contract_hash.detail contains fixture_shape=checked-arithmetic and b3: hash, support_boundary contains selected-fixture-only, non-proof boundary booleans remain false, and tampered/negative replay is rejected deterministically`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/mir-fixture-tests.txt`
- [x] [depends:I4] V3 Verify scenario "MIR rejects unsupported fixture shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'`; input=`cargo-octet/src/cli.rs`; expect=`PASS parser rejects branch-control-flow and call-return for MIR with deterministic expected-shape diagnostic and rejects missing --fixture-shape with requires explicit --fixture-shape checked-arithmetic`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/parser-tests.txt`
- [x] [depends:I4] V4 Verify scenario "MIR fixture scope rejects non-MIR family pairings" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'`; input=`cargo-octet/src/cli.rs`; expect=`PASS parser rejects rustc-mir-fixture paired with hir or thir and reports MIR-only pairing`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/parser-tests.txt`
- [x] [depends:I4] V5 Verify scenario "MIR default contract remains unsupported-planned after fixture expansion" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet supplement -- --nocapture && cargo test -p cargo-octet object_corpus_contract -- --nocapture'`; input=`docs/contributor-guide.md docs/content-addressed-rust-evidence-boundaries.md cargo-octet/src/object.rs`; expect=`PASS docs/default contract assert MIR unsupported-planned outside explicit selected fixture path`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/supplement-regression-tests.txt`
- [x] [depends:V1,V2,V3,V4,V5] V6 Verify aggregate Valence evidence gate with command=`nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; input=`flake.nix checks.x86_64-linux.valence-object-evidence`; expect=`aggregate gate includes MIR fixture proof and exits 0`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/valence-object-evidence.txt`
- [x] [depends:V6] V7 Verify active OpenSpec validity with command=`openspec validate add-mir-fixture-contract --strict --json`; input=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract`; expect=`change valid with no failures`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/openspec-validate.txt`
- [x] [depends:V7] V8 Verify all-spec validity with command=`openspec validate --all --strict --json`; input=`openspec/specs and openspec/changes/archive/2026-05-22-add-mir-fixture-contract`; expect=`all specs and active change pass`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/openspec-validate-all.txt`
- [x] [depends:V8] V9 Verify whitespace hygiene with command=`git diff --check`; input=`working tree diff`; expect=`no whitespace errors`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/git-diff-check.txt`

## Phase 3: Closeout checkpoints

- [x] [depends:H3,V9] H4 Human/oracle checkpoint: command=`done-review evidence bundle plus strict reviewer pass before archive`; input=`completed implementation diff and active change evidence`; asserts=`done-review finds no blocking MIR overclaim or verification omission`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/done-review.txt`
- [x] [depends:H4] I5 Sync delta spec into the main spec and archive the completed change.
- [x] [depends:I5] V10 Verify post-archive all-spec validity and whitespace hygiene with command=`openspec validate --all --strict --json && git diff --check`; input=`openspec/specs and openspec/changes/archive/2026-05-22-add-mir-fixture-contract`; expect=`all specs pass and no whitespace errors after archive`; evidence=`openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/post-archive-validate.txt`

## Change Status
- Status: archived
- Open tasks: none

## Verification Coverage

- `Scenario: MIR fixture inventory lists selected shapes` -> `V1` -> `openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/mir-fixture-tests.txt`
- `Scenario: MIR checked-arithmetic fixture emits pilot-supported evidence` -> `V2` -> `openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/mir-fixture-tests.txt`
- `Scenario: MIR rejects unsupported fixture shapes` -> `V3` -> `openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/parser-tests.txt`
- `Scenario: MIR fixture scope rejects non-MIR family pairings` -> `V4` -> `openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/parser-tests.txt`
- `Scenario: MIR default contract remains unsupported-planned after fixture expansion` -> `V5` -> `openspec/changes/archive/2026-05-22-add-mir-fixture-contract/evidence/supplement-regression-tests.txt`
