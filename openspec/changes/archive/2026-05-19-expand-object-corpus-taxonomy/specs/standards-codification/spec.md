## ADDED Requirements

### Requirement: Expanded internal object corpus taxonomy

Octet MUST maintain an expanded internal object corpus taxonomy that selects stable in-repository Rust source modules beyond `cargo-octet/src/*.rs`, records the selected paths in a checked-in fixture, and validates a deterministic object corpus receipt for that expanded source set while preserving bounded evidence claims.

#### Scenario: Expanded taxonomy names selected source buckets

- GIVEN the expanded internal object corpus taxonomy fixture
- WHEN the taxonomy is inspected by tests
- THEN it names the selected source paths from both the `cargo-octet` and `slotcar-standards` buckets
- AND every selected path is present in the expanded corpus receipt fixture.

#### Scenario: Expanded corpus fixture replays cleanly

- GIVEN the expanded internal object corpus receipt fixture
- WHEN Octet checks it against the taxonomy-selected source paths
- THEN the check report status is `match`
- AND every compared field matches.

#### Scenario: Expanded corpus preserves bounded evidence claims

- GIVEN the expanded internal object corpus receipt fixture
- WHEN tests inspect its boundary fields
- THEN `claims_correctness` is false
- AND `claims_semantic_equivalence` is false.
