## Design

1. Add a `VerifierFixture` DTO for external verifier-inspired examples. Fields include fixture id, source artifact ref, syntax family, expected verdict, expected diagnostics, assumptions, unsupported-feature labels, verifier-family label, fixture role, and non-claims.
2. Support fixture roles for happy-path proof, counterexample, malformed contract, unsupported feature, stale expectation, and false-verification guard.
3. Keep normalization pure over already-loaded source snippets and metadata. No external verifier execution is required for this rail.
4. Seed the matrix with categories inspired by Zani/Kani harnesses, Sunder/Creusot contracts, and Certus/Verus-style proof macros, but store only Octet-owned minimal fixtures.
5. Add negative fixtures that deliberately try to turn fixture acceptance into verifier-soundness, feature-parity, or proof-discharge claims.
6. Wire selected fixtures into proof-card/standards/UI tests only after normalization and claim-boundary validation pass.

## Non-Goals

- No vendoring or depending on zani, sunder, certus, Kani forks, or z4.
- No execution of external verifiers in starter checks.
- No claim that a fixture accepted by Octet has been proven by its source verifier or that Octet matches source verifier semantics.