## ADDED Requirements

### Requirement: Object corpus fixture freshness rail

The system MUST expose a named object-corpus fixture freshness rail that verifies checked-in object-corpus fixtures still match freshly generated receipts and reports.

#### Scenario: Named fixture freshness rail passes

- GIVEN the checked-in internal corpus receipt, expanded internal corpus receipt, and expanded caveat taxonomy fixtures
- WHEN an operator runs `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L`
- THEN the rail runs the focused fresh-fixture parity tests and fails if any checked-in fixture is stale.

#### Scenario: Aggregate Valence gate names freshness rail

- GIVEN the aggregate `valence-object-evidence` gate
- WHEN it succeeds
- THEN it requires the named object-corpus fixture freshness rail and prints `valence_object_evidence.object_corpus_fixture_freshness_gate: PASS`.

#### Scenario: Guidance names freshness obligation

- GIVEN contributor and OpenSpec task-template guidance for Valence object-evidence work
- WHEN a change touches object-corpus inputs such as `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, THIR/MIR fixture inventories, or object-corpus fixture paths
- THEN the guidance names the fixture regeneration obligation and the named freshness rail before closeout.
