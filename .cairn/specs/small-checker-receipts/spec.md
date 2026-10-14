# Small Checker Receipts Specification

## Purpose

Defines the `small-checker-receipts` capability.

## Requirements

### Requirement: Promotion-grade receipts distinguish producer and checker

r[octet.small_checker_receipts.producer_checker] Octet SHOULD distinguish the producer that emitted evidence material from the checker that validates promotion-grade or formal-proof claims when those roles are distinct.

#### Scenario: Valid producer/checker receipt passes

r[octet.small_checker_receipts.producer_checker.valid]

- GIVEN a promotion-grade evidence receipt names a producer identity, checker identity, checker version or artifact identity, checked scope, accepted facts, input bindings, output artifact identity, evidence tier, and non-claims
- WHEN receipt validation runs
- THEN validation SHOULD accept the receipt only if the checker identity is registered and the checked bindings match the referenced artifacts.
- AND the report SHOULD show both producer and checker identities without treating either identity alone as proof of correctness.

#### Scenario: Self-asserted producer claim cannot promote

r[octet.small_checker_receipts.producer_checker.self_asserted]

- GIVEN evidence material names only the producer or asks to use producer output as checker-validated promotion evidence without a registered checker result
- WHEN promotion-grade receipt or provenance validation runs
- THEN validation MUST fail closed or keep the evidence at its lower declared tier.
- AND Octet MUST NOT emit `formal_proof` or checker-validated provenance for the self-asserted claim.

### Requirement: Small checkers are deterministic pure cores

r[octet.small_checker_receipts.deterministic_checker] Receipt checker logic SHOULD be implemented as small deterministic cores that validate normalized in-memory inputs and return accepted facts, diagnostics, and non-claims.

#### Scenario: Checker accepts normalized inputs only

r[octet.small_checker_receipts.deterministic_checker.normalized]

- GIVEN a checker validates receipt bindings, artifact hashes, source scopes, evidence tiers, and non-claims
- WHEN reviewers inspect the checker core
- THEN the core SHOULD operate over typed normalized inputs supplied by a shell.
- AND filesystem traversal, process execution, environment reads, clocks, randomness, network access, logging, and output rendering SHOULD remain outside the core.

#### Scenario: Checker fails closed on malformed evidence

r[octet.small_checker_receipts.deterministic_checker.fail_closed]

- GIVEN a receipt has stale hashes, missing bindings, unsupported schema versions, malformed checker identity, unknown producer identity, unknown checker identity, invalid evidence tier, or overbroad claim wording
- WHEN the checker core runs
- THEN it MUST return deterministic diagnostics naming the rejected field or binding.

### Requirement: Checker registries and contracts stay fresh

r[octet.small_checker_receipts.registry] Checker IDs, promotion-grade receipt families, and generated contracts MUST be registered with ownership, generation, validation, positive-fixture, and negative-fixture metadata.

#### Scenario: Registry freshness covers checker receipts

r[octet.small_checker_receipts.registry.fresh]

- GIVEN Octet emits or validates checker-backed receipts
- WHEN policy freshness validation runs
- THEN the registry MUST identify the Rust owner source, generated contract artifact, generation command, validation command, registered checker IDs, and positive and negative fixtures.
- AND raw verifier internals, raw logs, raw environment state, and secrets MUST remain excluded except as bounded paths, labels, digests, allowlists, or explicit non-candidate rationale.

### Requirement: Small-checker boundaries are explicit

r[octet.small_checker_receipts.boundaries] Small-checker receipt validation MUST NOT claim external tool soundness, solver soundness, complete test adequacy, whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or verified-runtime behavior unless separate reviewed evidence supports that claim.

#### Scenario: Checker report preserves non-claims

r[octet.small_checker_receipts.boundaries.non_claim]

- GIVEN a checker-backed receipt validates
- WHEN Octet renders human, JSON, SARIF, proof-card, or provenance output
- THEN the output MUST state or encode the checked scope, accepted facts, and non-claims.
- AND validation success MUST remain scoped to the checker contract and matched artifacts.
