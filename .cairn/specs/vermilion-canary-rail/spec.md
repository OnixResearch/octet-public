# Vermilion Canary Rail Specification

## Purpose

Defines the `vermilion-canary-rail` capability.

## Requirements

### Requirement: Measured canary is recorded-only evidence

r[octet.vermilion_canary_rail.recorded_only] `cargo octet prove` MUST admit the
measured Vermilion u64 canary through a recorded-only rail. The rail MUST
validate the exact canary bytes, reviewed toolchain pins, source BLAKE3,
obligation counts, outcome, and non-claims. It MUST emit a manual-review proof
card with no formal-proof envelope and MUST preserve the recorded-only tier
through artifact verification and Valence export. It MUST NOT promote the
canary or the synthetic typed fixture to formal proof.

#### Scenario: Measured canary is validated
- GIVEN the exact measured Vermilion u64 canary receipt
- WHEN the recorded-only rail runs
- THEN it MUST validate the pinned toolchain, source, obligations, and non-claims
- AND it MUST emit a `manual_review_required` proof card with no formal envelope.

#### Scenario: Canary drift fails closed
- GIVEN a canary with changed source, toolchain, outcome, or non-claim bytes
- WHEN validation runs
- THEN it MUST reject the receipt
- AND it MUST NOT emit a passing proof card.

#### Scenario: Tiers survive artifact verification and Valence export
- GIVEN a recorded-only canary artifact bundle
- WHEN artifact verification and Valence export run
- THEN the canary card and provenance MUST remain `manual_review_required`
- AND no `formal_proof` record MUST be emitted.
