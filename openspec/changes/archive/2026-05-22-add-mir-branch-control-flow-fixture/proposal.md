## Why

THIR selected-fixture evidence now includes `branch-control-flow`, while MIR remains limited to `checked-arithmetic` and `call-return`. Adding a bounded MIR `branch-control-flow` selected fixture closes that asymmetric evidence gap without broadening default/non-fixture MIR behavior.

## What Changes

- Add checked-in MIR fixture `branch-control-flow` that maps to `mir-spike/branch_control_flow.rs::choose_branch`.
- Extend MIR CLI selector, runtime fixture inventory, contract metadata, docs, and selected-fixture tests.
- Capture focused MIR positive replay plus inventory, freshness, aggregate, and boundary evidence.

## Non-Goals

- No general MIR extraction support.
- No default/non-fixture MIR behavior change; it remains `unsupported-planned`.
- No correctness, semantic-equivalence, cross-toolchain-stability, pure-cache, or proof claim.
- No THIR inventory changes.

## Impact

- Files: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, MIR fixture files, object-corpus fixtures, docs, and standards spec.
- APIs: extends the explicit `--family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow` selected-fixture selector only.
- Testing: focused MIR fixture tests, inventory gates, object-corpus fixture freshness, aggregate Valence evidence, boundary checker, OpenSpec validation, and whitespace checks.
