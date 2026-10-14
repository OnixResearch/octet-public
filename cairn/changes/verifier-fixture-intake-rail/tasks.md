## Tasks

- [ ] [serial] r[octet.verifier_fixture_intake.profile] Define verifier-fixture DTOs and normalized fixture roles.
- [ ] [depends:verifier-fixture-profile] r[octet.verifier_fixture_intake.validation] Validate fixture identity, syntax family, expected verdict, diagnostics, assumptions, unsupported-feature labels, role, and non-claims.
- [ ] [depends:verifier-fixture-validation] r[octet.verifier_fixture_intake.positive] Add positive fixtures for bounds harnesses, contracts, loop invariants, mutable-borrow summaries, and proof-style assertions.
- [ ] [depends:verifier-fixture-validation] r[octet.verifier_fixture_intake.negative] Add negative fixtures for contradictory postconditions, unsupported features, malformed contracts, stale expectations, and verifier-soundness overclaims.
- [ ] [depends:verifier-fixture-fixtures] r[octet.verifier_fixture_intake.integration] Wire selected normalized fixtures into proof-card, standards, or UI regression tests without invoking external verifiers.
- [ ] [depends:verifier-fixture-integration] r[octet.verifier_fixture_intake.final_validation] Run fixture normalization tests, proof-card/standards/UI focused tests, artifact verification, Cairn gates, and focused Octet checks.