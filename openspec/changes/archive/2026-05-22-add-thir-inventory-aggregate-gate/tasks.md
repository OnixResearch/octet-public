## Phase 1: Implementation

- [x] [serial] I1 Add focused THIR supplement inventory Nix check and compose it into `valence-object-evidence`.
- [x] [serial] I2 Add/adjust integration assertions proving aggregate THIR inventory boundaries and default `unsupported-planned` metadata.
- [x] [serial] I3 Update contributor docs and OpenSpec delta for aggregate gate composition and non-proof boundaries.

## Phase 2: Deterministic verification

- [x] [depends:I1,I2] V1 Verify THIR inventory integration rail with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs`; expect=`PASS selected shapes, command roundtrip, branch-control-flow exclusion, unsupported-planned boundary`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/thir-inventory-tests.txt`
- [x] [depends:I1,V1] V2 Verify focused Nix rail with command=`nix build .#checks.x86_64-linux.thir-supplement-inventory-gate --no-link -L`; input=`flake.nix`; expect=`PASS focused THIR supplement inventory gate`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/thir-inventory-nix-gate.txt`
- [x] [depends:I1,V2] V3 Verify aggregate gate with command=`nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; input=`flake.nix`; expect=`PASS aggregate output includes thir_supplement_inventory_gate`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/valence-object-evidence.txt`
- [x] [depends:I3] V4 Verify OpenSpec validation with command=`openspec validate add-thir-inventory-aggregate-gate --strict --json && openspec validate --all --strict --json`; input=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate`; expect=`PASS active change and all specs`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/openspec-validate.txt`
- [x] [depends:V4] V5 Verify diff hygiene with command=`git diff --check`; input=`working tree diff`; expect=`exit 0`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/git-diff-check.txt`

## Phase 3: Human/oracle checkpoints

- [x] [depends:V1,V2,V3,V4,V5] H1 Human/oracle checkpoint: command=`pi --no-session -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call the openspec_gate tool for change add-thir-inventory-aggregate-gate at tasks stage. Print the full result."`; input=`proposal/design/tasks/spec delta`; asserts=`tasks gate PASS`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/tasks-gate.txt`
- [x] [depends:H1] H2 Human/oracle checkpoint: command=`delegate_task strict done-review with review_gate_evidence`; input=`post-archive diff and evidence`; asserts=`review finds no blockers`; evidence=`openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/done-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Aggregate gate composes THIR inventory rail` -> `V2`, `V3` -> `openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/thir-inventory-nix-gate.txt`, `openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/valence-object-evidence.txt`
- `Scenario: Aggregate docs preserve THIR inventory boundaries` -> `V1`, `V4` -> `openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/thir-inventory-tests.txt`, `openspec/changes/archive/2026-05-22-add-thir-inventory-aggregate-gate/evidence/openspec-validate.txt`
