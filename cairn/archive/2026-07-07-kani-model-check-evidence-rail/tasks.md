## Phase 1: Rail registration and receipt model

- [x] [serial] r[octet.kani_evidence_rail.optional_collection] Register a Kani model-check external evidence rail that is opt-in and excluded from starter `cargo octet check`.
- [x] [serial] r[octet.kani_evidence_rail.receipt_identity] Define Rust-owned receipt DTOs and generated Nickel contract coverage for harness identity, toolchain identity, solver, unwind, contract, stub, verdict, and hash fields.
- [x] [serial] r[octet.kani_evidence_rail.classification] Add evidence classification logic for bounded harnesses, passing unwinding assertions, contract proofs, verified stubs, and plain stubs.

## Phase 2: Collection, verification, and fixtures

- [x] [parallel] r[octet.kani_evidence_rail.collection_failure_modes] Implement fail-closed receipt validation for missing identity, stale inputs, failed checks, failed unwinding assertions, missing non-claims, and unsupported promotion.
- [x] [parallel] r[octet.kani_evidence_rail.fixture_coverage] Add positive fixtures for bounded harness, contract harness, and verified-stub receipt shapes.
- [x] [parallel] r[octet.kani_evidence_rail.fixture_coverage] Add negative fixtures for missing tool identity, Kani failure, unwind failure, stale input hash, missing non-claim, and plain-stub promotion.

## Phase 3: Documentation and validation

- [x] [serial] r[octet.kani_evidence_rail.boundary_documentation] Document the Kani rail scope, command shape, receipt fields, and non-claims for operators.
- [x] [serial] r[octet.kani_evidence_rail.optional_collection] Run artifact verification tests, rail fixture tests, policy freshness checks, and Cairn validation/gates before archive.

## Verification Coverage

| Scenario | Planned Evidence |
|---|---|
| Passing bounded harness receipt is accepted | Positive fixture test |
| Passing contract harness receipt is accepted | Positive fixture test |
| Verified-stub link is accepted only with contract receipt | Positive and negative fixture tests |
| Missing tool identity fails closed | Negative fixture test |
| Failed Kani verdict fails closed for passing evidence | Negative fixture test |
| Failed unwinding assertion is not unbounded evidence | Negative fixture test |
| Plain-stub promotion fails closed | Negative fixture test |
| Operator boundary is visible | Docs check or transcript |
