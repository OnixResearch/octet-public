# Tasks: add-thir-fixture-contract

## Implementation

- [x] I1 [serial] Add the THIR fixture scope, closed selector `checked-arithmetic`, exact fixture/function binding `cargo-octet/tests/fixtures/thir-spike/checked.rs::checked`, and fixture-backed `pilot-supported` supplement constructor.
- [x] I2 [serial] Update CLI parser/help, fixture inventory, contract metadata, and docs so THIR selected-fixture support is distinct from default unsupported-planned behavior; metadata must name `selected_fixture_shapes=["checked-arithmetic"]` and the exact fixture replay command.
- [x] I3 [serial] Add positive and negative tests for THIR fixture emit/check, parser/help coverage, contract metadata, exact observation keys `fixture_context`, `thir_fixture_contract_hash`, `typed_expression_shape`, `support_boundary`, and non-proof boundaries.

## Verification

- [x] V1 [depends:I1,I2,I3] Verify selected THIR fixture support.
  - scenario: `THIR fixture emits pilot-supported evidence`
  - command=`TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet thir_fixture -- --nocapture'`
  - input=`cargo-octet/tests/fixtures/thir-spike/checked.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`
  - expect=`pilot-supported` only for `--family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic`; boundary claims remain false
  - evidence=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/thir-fixture-tests.txt`

- [x] V2 [depends:I1,I2,I3] Verify supplement regression and metadata coverage.
  - scenario: `THIR fixture metadata remains bounded`
  - command=`TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet supplement -- --nocapture && cargo test -p cargo-octet object_corpus_contract -- --nocapture'`
  - input=`cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`
  - expect=HIR/MIR supplement behavior still passes; default THIR remains `unsupported-planned`; contract metadata names the selected THIR fixture shape and non-proof boundaries
  - evidence=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/supplement-regression-tests.txt`

- [x] V3 [depends:I1,I2,I3] Verify OpenSpec and whitespace readiness.
  - scenario: `OpenSpec package and changed files are ready to archive`
  - command=`openspec validate add-thir-fixture-contract --strict --json && git diff --check`
  - input=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/`, changed Rust/docs/spec files
  - expect=strict validation passes and whitespace check has no findings
  - evidence=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/openspec-validate.txt`, `openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/git-diff-check.txt`

- [x] V4 [depends:V1,V2,V3] Verify proposal/design/tasks gates.
  - scenario: `OpenSpec gates accept the bounded THIR fixture package`
  - command=`pi --no-session --thinking minimal -e ~/.pi/agent/extensions/openspec-gates.ts -p 'Call the openspec_gate tool for stage=<stage> and change=add-thir-fixture-contract in /path/to/git/octet. Print the full result.'`
  - input=`stage=proposal`, `stage=design`, `stage=tasks`
  - expect=all three gates return PASS/accepted without critical findings
  - evidence=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/proposal-gate.txt`, `design-gate.txt`, `tasks-gate.txt`

## Archive / Closeout

- [x] H1 [depends:V1,V2,V3,V4] Archive the completed OpenSpec change.
  - command=`openspec archive add-thir-fixture-contract -y`
  - input=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/`
  - asserts=archive path exists under `openspec/changes/archive/2026-05-21-add-thir-fixture-contract/`; active change path is removed; archive evidence paths are rewritten
  - evidence=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/archive-command.txt`

- [x] H2 [depends:H1] Verify post-archive all-spec state and staged diff hygiene.
  - command=`openspec validate --all --strict --json && git diff --check`
  - input=archived change and canonical `openspec/specs/standards-codification/spec.md`
  - asserts=all specs pass strict validation; whitespace check has no findings; no stale active-change references remain except deliberate historical text none expected
  - evidence=`openspec/changes/archive/2026-05-21-add-thir-fixture-contract/evidence/openspec-validate-all.txt`, `git-diff-check.txt`

## Verification Coverage

| Scenario | Tasks | Evidence |
| --- | --- | --- |
| `THIR fixture emits pilot-supported evidence` | I1, I3, V1 | `evidence/thir-fixture-tests.txt` |
| `THIR fixture check rejects mismatched receipts` | I1, I3, V1 | `evidence/thir-fixture-tests.txt` |
| `Default THIR remains unsupported-planned` | I1, I2, I3, V2 | `evidence/supplement-regression-tests.txt` |
| `THIR fixture metadata remains bounded` | I2, I3, V2 | `evidence/supplement-regression-tests.txt` |
| OpenSpec package and archive hygiene | V3, V4, H1, H2 | `evidence/openspec-validate.txt`, gate evidence, archive closeout evidence |
