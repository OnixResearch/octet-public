# Product Positioning Specification

## Purpose

Defines the `product-positioning` capability.

## Requirements

### Requirement: Agent checked-path positioning

r[octet.product_positioning.agent_checked_path] Consumer-facing Octet documentation MUST describe Octet as an evidence-backed checked path for agent-written and human-written Rust systems code.

#### Scenario: Reader sees the checked path

r[octet.product_positioning.agent_checked_path.visible]

- GIVEN a reader opens the README or consumer guide
- WHEN they read the product introduction
- THEN they MUST see that Octet routes Rust source through checks, policy/config, evidence artifacts, and CI or release gates.
- AND the path MUST name at least one concrete Octet command or artifact surface.

### Requirement: No verified-runtime overclaim

r[octet.product_positioning.no_verified_runtime_claim] Octet positioning MUST NOT claim that Octet is a verified runtime or that it proves whole-program correctness unless the claim is explicitly scoped to a specific verified artifact.

#### Scenario: Forbidden product claim is absent

r[octet.product_positioning.no_verified_runtime_claim.absent]

- GIVEN consumer-facing Octet documentation is scanned
- WHEN the scan encounters language about verification or proof
- THEN any claim MUST be scoped to the concrete check, proof artifact, or evidence tier that supports it.
- AND unqualified claims that Octet is a verified runtime or proves program correctness MUST fail documentation validation.
