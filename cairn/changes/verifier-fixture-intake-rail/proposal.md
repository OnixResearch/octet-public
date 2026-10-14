## Why

Octet benefits from adversarial verifier fixtures that exercise proof-rail UX, non-claim boundaries, and false-verification failure modes without adopting every verifier as a dependency. The zani, sunder, and certus repositories are useful references for Kani-compatible harnesses, Creusot-style contracts, Verus-style proof ergonomics, mutable-borrow reasoning, and false-verified fuzzing patterns.

Reference sources: `https://github.com/joweeba/zani`, `https://github.com/joweeba/sunder`, and `https://github.com/joweeba/certus`.

## What Changes

- Define a verifier-fixture intake rail for normalized positive and negative verifier examples.
- Record fixture source identity, syntax family, expected verdict, failure mode, verifier-family label, assumptions, and non-claims.
- Add an initial fixture matrix for bounds checks, contracts, contradictory postconditions, mutable borrow summaries, loop invariants, and unsupported-feature diagnostics.
- Keep fixtures as test/evidence inputs only; Octet does not adopt those verifier runtimes or claim parity.

## Impact

- **Surfaces**: proof artifact library, Kani/Verus rail fixture corpus, standards proof-card docs, UI/regression tests.
- **Non-claims**: No Zani/Sunder/Certus dependency, no verifier parity claim, no proof soundness transfer.
- **Validation**: fixture normalization tests, positive/negative rail tests, artifact verification, Cairn gates/validate.