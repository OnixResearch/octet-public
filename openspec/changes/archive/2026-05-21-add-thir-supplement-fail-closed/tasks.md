## Phase 1: Implementation

- [x] [serial] I1 Extend supplement family parsing and runtime DTO support with fail-closed THIR records.
- [x] [depends:I1] I2 Add focused runtime tests for THIR emit/check behavior without broadening HIR fixture inventory.

## Phase 2: Deterministic verification

- [x] [depends:I2] V1 Verify scenario "THIR supplement emits unsupported-planned evidence" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_supplement -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs`; expect=`PASS thir emit assertions for schema/status/boundary`; evidence=`openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/thir-supplement-tests.txt`
- [x] [depends:V1] V2 Verify scenario "THIR supplement check replays fail-closed receipt" with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_supplement -- --nocapture'`; input=`cargo-octet/tests/fingerprint_consumer.rs`; expect=`PASS thir check replay assertions for status=pass and non-proof boundary`; evidence=`openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/thir-supplement-tests.txt`
- [x] [depends:V2] V3 Verify OpenSpec validity with command=`openspec validate add-thir-supplement-fail-closed --strict --json`; input=`openspec/changes/add-thir-supplement-fail-closed`; expect=`change valid with no failures`; evidence=`openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/openspec-validate.txt`

## Phase 3: Closeout

- [x] [depends:V3] I3 Sync delta spec into the main spec and archive the completed change.
- [x] [depends:I3] V4 Verify final all-spec validity with command=`openspec validate --all --strict --json`; input=`openspec/specs and archived change`; expect=`all specs pass`; evidence=`openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/openspec-validate-all.txt`
- [x] [depends:I3] V5 Verify whitespace with command=`git diff --check`; input=`working tree diff`; expect=`no whitespace errors`; evidence=`openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/git-diff-check.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: THIR supplement emits unsupported-planned evidence` -> `V1` -> `openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/thir-supplement-tests.txt`
- `Scenario: THIR supplement check replays fail-closed receipt` -> `V2` -> `openspec/changes/archive/2026-05-21-add-thir-supplement-fail-closed/evidence/thir-supplement-tests.txt`
