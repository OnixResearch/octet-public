## Phase 1: Implementation

- [x] I1 Add Rust-derived HIR/MIR supplemental evidence metadata to the object corpus contract report.
- [x] I2 Add tests asserting JSON and human metadata fail closed and preserve non-proof boundaries.

## Phase 2: Verification

- [x] V1 Run focused contract metadata tests and save evidence.
- [x] V2 Run full Octet verification and save evidence.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cargo octet object corpus contract --output-format json`; input=`object corpus contract metadata`; asserts=`HIR/MIR families are reported as unsupported-planned, supplemental, toolchain-scoped, and non-proof`; evidence=`openspec/changes/archive/2026-05-19-hir-mir-supplement-contract-metadata/evidence/contract-metadata.txt`

## Verification Coverage

- `Scenario: Contract metadata names planned HIR/MIR supplements` -> `I1`/`I2`/`V1` -> `evidence/contract-metadata-tests.txt`
- `Scenario: Planned supplements fail closed before extractor implementation` -> `I1`/`I2`/`V1` -> `evidence/contract-metadata-tests.txt`
- `Scenario: Human contract output preserves boundaries` -> `I2`/`V1`/`H1` -> `evidence/contract-metadata.txt`
