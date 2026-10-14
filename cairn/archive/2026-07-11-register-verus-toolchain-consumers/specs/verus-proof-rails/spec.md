# Verus Proof Rails Specification Delta

## ADDED Requirements

### Requirement: Exact package consumers are registered explicitly [r[octet.verus_proof_rails.toolchain_profile.consumer_registration]]

Octet MUST register every reviewed downstream consumer of its exact Verus toolchain packages in the typed compatibility inventory. Consumer identifiers MUST come from the reviewed closed set, and each row MUST state whether the relationship is native, locally pinned, semantic-reference-only, or unsupported.

A native consumer row MUST mean only that the consumer selects the exact Octet package profile. It MUST NOT transfer proof validity, discharge consumer-specific assumptions, or authorize runtime or release promotion.

#### Scenario: Exact downstream package consumer is admitted

- GIVEN OnixOS lifecycle proof execution or Aspen's node-replication probe selects the exact Octet profile and packages
- WHEN the compatibility profile is validated
- THEN the consumer MUST have a reviewed `native` row with a bounded rationale
- AND the row MUST preserve consumer-owned proof and readiness decisions

#### Scenario: Unknown consumer is rejected

- GIVEN a compatibility row uses an identifier outside the reviewed consumer inventory
- WHEN Nickel contract validation runs
- THEN profile export MUST fail before package or proof execution
- AND the unknown row MUST NOT inherit authority from a matching version string
