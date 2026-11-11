# Tasks

## Implementation

- [x] I1 Add the MIR `branch-control-flow` fixture file and selector/runtime mapping.
- [x] I2 Update MIR fixture inventory, contract metadata, CLI diagnostics/tests, and selected-fixture replay tests.
- [x] I3 Update docs/spec wording while preserving selected-fixture-only, default `unsupported-planned`, and non-proof boundaries.

## Verification

- [x] V1 Run focused Rust tests for MIR fixture replay/inventory and THIR inventory preservation.
- [x] V2 Run object-corpus fixture freshness and refresh checked-in object-corpus fixtures if needed.
- [x] V3 Run MIR inventory and aggregate Valence Nix rails.
- [x] V4 Run boundary checker, OpenSpec validation, and whitespace checks.

## Human Evidence

- [x] H1 Record human boundary review with command=manual review, input=changed MIR/THIR/docs/spec surfaces, asserts=selected-fixture-only/no general MIR/no proof/default unsupported-planned, evidence=evidence/human-boundary-review.txt.

## Verification Coverage

| Scenario | Verification | Evidence |
| --- | --- | --- |
| MIR branch-control-flow emits/checks as selected fixture | V1, V3 | evidence/mir-branch-control-flow-tests.txt; evidence/mir-inventory-nix-gate.txt |
| MIR inventory is exact and replayable | V1, V3 | evidence/mir-branch-control-flow-tests.txt; evidence/mir-inventory-nix-gate.txt |
| Default/non-fixture MIR remains unsupported-planned and non-proof | V1, V4, H1 | evidence/boundary-check.txt; evidence/human-boundary-review.txt |
| THIR inventory remains unchanged | V1, V4, H1 | evidence/thir-inventory-preservation.txt; evidence/human-boundary-review.txt |
| Object-corpus fixtures remain fresh after source changes | V2 | evidence/object-corpus-fixture-freshness.txt |
| Aggregate Valence evidence includes updated MIR inventory | V3 | evidence/valence-object-evidence.txt |
