## MODIFIED Requirements

### Requirement: Function Object Corpus Receipt Check [r[function-object-corpus-receipt-check]]
Object corpus receipt checks MUST preserve named compatibility fixture coverage for accepted v1 receipts and fail-closed malformed/tampered receipt or source-selection examples without changing the v1 receipt schema.

#### Scenario: Boundary tamper fixture fails closed
- GIVEN a checked-in v1 receipt fixture whose `boundary.claims_correctness` value is tampered to `true`
- WHEN `cargo octet object corpus check` replays it against the compatibility source fixture
- THEN the check MUST fail closed with a `mismatch` report naming the `boundary` comparison field.

#### Scenario: Duplicate receipt source path fixture fails closed
- GIVEN a checked-in v1 receipt fixture whose `source_paths` array repeats the same source file
- WHEN `cargo octet object corpus check` validates it
- THEN the check MUST fail closed with an `invalid` report containing `duplicate-source-path` for `source_paths`.

#### Scenario: Malformed receipt fixture fails closed
- GIVEN a checked-in malformed receipt JSON fixture
- WHEN `cargo octet object corpus check` validates it
- THEN the check MUST fail closed with an `invalid` report containing `invalid-json` for `receipt`.

#### Scenario: Empty receipt source path fixture fails closed
- GIVEN a checked-in v1 receipt fixture whose `source_paths` array is empty
- WHEN `cargo octet object corpus check` validates it against the compatibility source fixture
- THEN the check MUST fail closed with an `invalid` report containing `empty-source-selection` for `source_paths`.

#### Scenario: Unsupported non-Rust source selection fixture fails closed
- GIVEN the checked-in positive v1 compatibility receipt and a checked-in non-Rust source-selection fixture
- WHEN `cargo octet object corpus check` is invoked with that non-Rust source path
- THEN the check MUST fail closed with an `invalid` report containing `unsupported-source-path` for `paths`.
