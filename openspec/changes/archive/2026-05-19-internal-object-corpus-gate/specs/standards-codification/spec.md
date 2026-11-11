## ADDED Requirements

### Requirement: Internal Object Corpus Gate

The system MUST provide a checked-in gate fixture and deterministic replay test for Octet's bounded internal function-object corpus.

The gate MUST compare the saved receipt against current selected source inputs using `cargo octet object corpus check --output-format json`, and it MUST preserve the boundary that matching object corpus receipts validate review/provenance evidence identity only.

#### Scenario: Checked-in internal corpus receipt replays cleanly

- GIVEN a checked-in internal object corpus receipt fixture for the bounded Octet source corpus
- WHEN the gate replays the fixture against current selected source paths
- THEN the gate exits successfully and every receipt-critical comparison reports `match`

#### Scenario: Internal corpus gate preserves evidence-only boundaries

- GIVEN the checked-in corpus receipt and replay report
- WHEN the gate inspects the boundary comparison
- THEN the report preserves `claims_correctness = false` and `claims_semantic_equivalence = false` without claiming behavioral correctness or semantic equivalence

#### Scenario: Fixture refresh is deterministic

- GIVEN the checked-in internal corpus receipt fixture
- WHEN a fresh receipt is generated from the same bounded source corpus
- THEN the fresh receipt bytes match the checked-in fixture or the review must include an intentional fixture update
