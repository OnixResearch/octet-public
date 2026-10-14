# Scope Boundary Specification

## Purpose

Defines the `scope-boundary` capability.

## Requirements

### Requirement: Unsupported claims are explicit

r[octet.scope_boundary.unsupported_claims] Octet documentation MUST explicitly state that Octet does not prove whole-program correctness, does not validate business or domain requirements, and does not make suppressions correct merely because they are present.

#### Scenario: Reader sees unsupported-claim boundary

r[octet.scope_boundary.unsupported_claims.visible]

- GIVEN a reader opens primary Octet documentation
- WHEN they inspect the scope-boundary section
- THEN they MUST see that Octet does not prove whole-program correctness.
- AND they MUST see that Octet does not validate business or domain requirements.
- AND they MUST see that Octet does not make suppressions correct merely because they are present.

#### Scenario: Unqualified guarantee fails validation

r[octet.scope_boundary.unsupported_claims.no_unqualified_guarantee]

- GIVEN documentation validation scans consumer-facing docs
- WHEN it encounters an unqualified claim that Octet proves correctness, guarantees safety, or validates business requirements
- THEN validation MUST fail with a diagnostic naming the unsupported claim.

### Requirement: Supported capabilities are paired with boundaries

r[octet.scope_boundary.supported_capabilities] Each unsupported-claim boundary SHOULD be paired with Octet's supported capabilities: static rejection of known-dangerous source shapes, policy validation, reviewable suppressions, evidence artifacts, and CI or release gates.

#### Scenario: Boundary names supported path

r[octet.scope_boundary.supported_capabilities.named]

- GIVEN the scope-boundary section lists unsupported claims
- WHEN a reader continues through the same section
- THEN it SHOULD name the supported Octet capabilities that remain in scope.
- AND it SHOULD link or point to the artifacts reviewers inspect.
