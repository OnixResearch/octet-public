## MODIFIED Requirements

### Requirement: Function Object Corpus Receipt Check [r[function-object-corpus-receipt-check]]

The system MUST support replaying a saved function-object corpus receipt against selected Rust source inputs and reporting whether the current evidence identity still matches the saved receipt.

The check MUST compare receipt-critical evidence fields, including schema, object count, selected source paths, object-set hash, caveat counts, pure-cache blocker count, and no-correctness/no-semantic-equivalence boundary fields. The check MUST fail closed on drift, malformed receipts, unsupported schema, unsupported schema versions, missing required fields, invalid source selections, empty corpora, or boundary tampering.

Invalid receipt or selection failures MUST emit a deterministic `object corpus check` report with `status = "invalid"` and typed errors when JSON or human output is requested. These failure reports MUST preserve the boundary that receipt checks validate review/provenance evidence identity only; they MUST NOT claim behavioral correctness, semantic equivalence, or cross-machine determinism.

#### Scenario: Saved receipt replays cleanly

- GIVEN a saved function-object corpus receipt for selected Rust source files
- WHEN the corpus check command is run with the same source inputs
- THEN the command exits successfully
- AND the report marks schema, object count, source paths, object-set hash, caveat counts, pure-cache blocker count, and boundary fields as matching

#### Scenario: Source drift fails the receipt check

- GIVEN a saved function-object corpus receipt
- WHEN a selected source function changes before the check is rerun
- THEN the command exits non-zero
- AND the report identifies at least the object-set hash drift without claiming semantic equivalence

#### Scenario: Boundary tampering fails the receipt check

- GIVEN a saved corpus receipt whose no-correctness or no-semantic-equivalence boundary was altered
- WHEN the corpus check command is run
- THEN the command exits non-zero
- AND the report identifies boundary drift

#### Scenario: Malformed receipt emits typed invalid report

- GIVEN a saved receipt file that is not valid JSON
- WHEN the corpus check command is run with JSON output
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with an `invalid-json` error kind

#### Scenario: Unsupported schema is rejected before comparison

- GIVEN a saved receipt with an unknown schema or future schema version
- WHEN the corpus check command is run
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with a typed schema error

#### Scenario: Missing receipt fields are rejected before comparison

- GIVEN a saved receipt missing a required critical field
- WHEN the corpus check command is run
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with a `missing-field` error naming the field

#### Scenario: Invalid source selection is rejected deterministically

- GIVEN a check invocation with missing, duplicate, non-Rust, or empty-corpus source selection
- WHEN the corpus check command is run with JSON output
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with typed source-selection errors
