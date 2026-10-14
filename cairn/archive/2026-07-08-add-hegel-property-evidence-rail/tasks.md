## Phase 1: Rail registration and receipt contract

- [x] [serial] r[octet.dynamic_evidence.hegel_property_rail] Register a Hegel property external evidence rail that is opt-in and excluded from starter `cargo octet check`.
- [x] [serial] r[octet.dynamic_evidence.hegel_property_receipt] Define Rust-owned Hegel receipt DTOs and generated Nickel contract coverage for property identity, runner settings, generator identity, status, counterexample identity, source scope, parent artifacts, hashes, and non-claims.
- [x] [serial] r[octet.dynamic_evidence.hegel_property_boundary] Add receipt classification that preserves Hegel as generated-testing evidence and rejects formal-proof or exhaustive-correctness promotion.

## Phase 2: Law-test integration and fixtures

- [x] [parallel] r[octet.law_tested_deterministic_cores.hegel] Add Hegel dev/test integration for selected pure Octet cores with named law IDs, named budgets, deterministic or replayable settings, and no shell effects in the core under test.
- [x] [parallel] r[octet.dynamic_evidence.hegel_property_receipt.fail_closed] Add positive and negative fixtures for complete receipts, missing identity, malformed BLAKE3, stale generator hash, failing property status, unsupported tier promotion, and missing non-claims.
- [x] [parallel] r[octet.law_tested_deterministic_cores.hegel.fixture_complement] Keep user-visible Dylint UI, CLI golden, transcript, and artifact-verification fixtures in place for surfaces covered by Hegel laws.

## Phase 3: Provenance, Valence export, and docs

- [x] [serial] r[octet.evidence_provenance.valence_export.hegel_receipts] Extend provenance producer/export validation so Hegel receipts can be exported to Valence as generated-testing evidence without tier promotion.
- [x] [serial] r[octet.dynamic_evidence.hegel_property_boundary.docs] Document rail command shape, receipt fields, replay/counterexample guidance, sandbox limitations, and non-claims.
- [x] [serial] r[octet.dynamic_evidence.hegel_property_rail] Run Hegel rail fixture tests, artifact verification tests, Valence export fixture tests, policy freshness checks, `cargo test --workspace`, and Cairn validation/gates.

## Verification Coverage

| Scenario | Planned Evidence |
|---|---|
| Hegel rail is opt-in and starter check is unaffected | CLI/unit test and docs check |
| Complete passing receipt validates | Positive receipt fixture |
| Missing identity/hash/non-claim fails closed | Negative receipt fixtures |
| Failing property cannot be presented as passing evidence | Negative fixture |
| Formal-proof or exhaustive-correctness promotion fails | Negative fixture |
| Pure-core Hegel law has replayable settings | Focused Hegel property test |
| User-visible behavior remains fixture-backed | UI/golden/artifact fixture check |
| Valence export preserves generated-testing tier | Positive and negative export fixtures |
