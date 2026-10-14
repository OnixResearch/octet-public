## Phase 1: Implementation

- [x] [serial] I1 Add explicit THIR fixture inventory unit coverage for exact selectors, paths, functions, statuses, boundaries, and HIR-only selector exclusion.
- [x] [serial] I2 Add integration coverage that consumes THIR inventory JSON and roundtrips emitted `emit_command`/`check_command` for every listed THIR shape.

## Phase 2: Deterministic verification

- [x] [depends:I1] V1 Verify scenario "THIR fixture inventory lists selected shapes" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_fixture_inventory -- --nocapture'`; input=`cargo-octet/src/object.rs`; expect=`PASS exact checked-arithmetic/call-return inventory and no branch-control-flow`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-fixture-inventory-tests.txt`
- [x] [depends:I2] V2 Verify scenario "THIR inventory commands roundtrip selected fixture evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory_commands_roundtrip -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs`; expect=`PASS emitted inventory commands replay for both THIR shapes`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-inventory-roundtrip-tests.txt`
- [x] [depends:I1] V3 Verify scenario "THIR fixture inventory preserves unsupported-planned family boundary" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_corpus_contract_metadata_reports_pilot_and_planned_supplement_statuses -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs`; expect=`PASS thir status unsupported-planned and selected_fixture_shapes checked-arithmetic/call-return`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-contract-boundary-tests.txt`
- [x] [depends:V1,V2,V3] V4 Verify change OpenSpec validation with command=`openspec validate add-thir-fixture-inventory-rail --strict --json && openspec validate --all --strict --json`; input=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail`; expect=`PASS active change and all specs`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/openspec-validate.txt`
- [x] [depends:V4] V5 Verify diff hygiene with command=`git diff --check`; input=`working tree diff`; expect=`exit 0 with no whitespace diagnostics`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/git-diff-check.txt`

## Phase 3: Human/oracle checkpoints

- [x] [depends:V1,V2,V3,V4,V5] H1 Human/oracle checkpoint: command=`pi --no-session -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call the openspec_gate tool for change add-thir-fixture-inventory-rail at tasks stage. Print the full result."`; input=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/proposal.md, design.md, tasks.md, specs/standards-codification/spec.md`; asserts=`tasks gate confirms implementation/verification tasks cover every modified THIR inventory scenario`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/tasks-gate.txt`
- [x] [depends:H1] H2 Human/oracle checkpoint: command=`delegate_task strict done-review with review_gate_evidence`; input=`active change diff and evidence`; asserts=`review finds no blocker before archiving the THIR fixture inventory rail`; evidence=`openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/done-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: THIR fixture inventory lists selected shapes` -> `V1` -> `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-fixture-inventory-tests.txt`
- `Scenario: THIR inventory commands roundtrip selected fixture evidence` -> `V2` -> `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-inventory-roundtrip-tests.txt`
- `Scenario: THIR fixture inventory preserves unsupported-planned family boundary` -> `V3` -> `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-contract-boundary-tests.txt`
