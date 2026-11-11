## Phase 1: Implementation

- [x] I1 Add checked-in THIR `branch-control-flow` fixture selector and runtime inventory mapping.
- [x] I2 Update focused THIR inventory/roundtrip tests and MIR fail-closed negative assertions.
- [x] I3 Update Valence evidence docs and docs boundary helper for THIR branch-control-flow non-claims.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "THIR fixture inventory lists selected branch-control-flow shape" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory -- --nocapture'`; input=`cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs`; expect=`THIR inventory lists exactly checked-arithmetic, call-return, branch-control-flow`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/thir-branch-control-flow-tests.txt`
- [x] V2 Verify scenario "THIR branch-control-flow commands roundtrip selected fixture evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory -- --nocapture'`; input=`cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs`; expect=`branch-control-flow emits pilot-supported and check status pass`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/thir-branch-control-flow-tests.txt`
- [x] V3 Verify scenario "MIR fixture inventory remains branch-control-flow closed" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`; input=`cargo-octet/tests/fixtures/mir-spike/`; expect=`MIR inventory remains checked-arithmetic plus call-return and branch-control-flow is rejected`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/mir-branch-control-flow-negative.txt`
- [x] V4 Verify scenario "Aggregate gate keeps THIR branch-control-flow inventory visible" with command=`nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L --rebuild`; input=`flake.nix`; expect=`valence_object_evidence.thir_supplement_inventory_gate: PASS`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/valence-object-evidence.txt`
- [x] V5 Verify scenario "Contributor docs name THIR branch-control-flow boundaries" with command=`python3 openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/check-doc-boundaries.py`; input=`docs/contributor-guide.md docs/current-head-valence-evidence.md`; expect=`doc_boundaries: PASS`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/doc-boundaries.txt`
- [x] V6 Verify final OpenSpec and whitespace gates with command=`openspec validate --all --strict --json && git diff --check`; input=`openspec/ docs/ cargo-octet/ flake.nix`; expect=`all OpenSpec items pass and git diff --check exits 0`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/final-validation.txt`
- [x] V7 Verify CLI parser/unit surfaces after fixture-selector changes with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet cli::tests -- --nocapture'`; input=`cargo-octet/src/cli.rs`; expect=`CLI parser tests pass and MIR no-shape diagnostics remain limited to MIR-selected fixtures`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/cli-tests.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`manual review of changed fixture/runtime/docs/evidence surfaces`; input=`cargo-octet/src/cli.rs cargo-octet/src/object.rs cargo-octet/tests/fingerprint_consumer.rs docs/contributor-guide.md docs/current-head-valence-evidence.md`; asserts=`THIR branch-control-flow is selected-fixture-only and does not imply general THIR extraction, correctness, semantic equivalence, or MIR branch-control-flow support`; evidence=`openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/human-boundary-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: THIR fixture inventory lists selected branch-control-flow shape` -> `V1` -> `openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/thir-branch-control-flow-tests.txt`
- `Scenario: THIR branch-control-flow commands roundtrip selected fixture evidence` -> `V2` -> `openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/thir-branch-control-flow-tests.txt`
- `Scenario: MIR fixture inventory remains branch-control-flow closed` -> `V3` -> `openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/mir-branch-control-flow-negative.txt`
- `Scenario: Aggregate gate keeps THIR branch-control-flow inventory visible` -> `V4` -> `openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/valence-object-evidence.txt`
- `Scenario: Contributor docs name THIR branch-control-flow boundaries` -> `V5` -> `openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/doc-boundaries.txt`
- CLI parser/unit regression guard -> `V7` -> `openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/cli-tests.txt`
