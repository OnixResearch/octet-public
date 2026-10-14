## ADDED Requirements

### Requirement: Function Object Corpus Receipt Check [r[function-object-corpus-receipt-check]]

The system MUST support replaying a saved function-object corpus receipt against selected Rust source inputs and reporting whether the current evidence identity still matches the saved receipt.

The check MUST compare receipt-critical evidence fields, including schema, object count, selected source paths, object-set hash, caveat counts, pure-cache blocker count, and no-correctness/no-semantic-equivalence boundary fields. The check MUST fail closed on drift, malformed receipts, unsupported schema, or boundary tampering.

#### Scenario: Saved corpus receipt replays cleanly

- GIVEN a saved corpus receipt emitted for a selected set of Rust source files
- WHEN the corpus receipt check runs against the same source files
- THEN the command exits successfully and reports `status = "match"`

#### Scenario: Source drift fails corpus receipt check

- GIVEN a saved corpus receipt emitted before a function body changes
- WHEN the corpus receipt check runs against the changed source files
- THEN the command exits non-zero and reports an object-set hash mismatch

#### Scenario: Boundary tamper fails corpus receipt check

- GIVEN a saved corpus receipt whose no-correctness or no-semantic-equivalence boundary was altered
- WHEN the corpus receipt check runs against the original source files
- THEN the command exits non-zero and reports a boundary mismatch
