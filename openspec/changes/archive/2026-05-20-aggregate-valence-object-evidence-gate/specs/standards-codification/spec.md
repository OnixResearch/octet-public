## ADDED Requirements

### Requirement: Aggregate Valence Object Evidence Gate [r[aggregate-valence-object-evidence-gate]]

The system MUST provide a named Nix check that aggregates Octet's selected Valence object-evidence rails for operator and CI use.

The aggregate gate MUST compose the checked-in internal object corpus replay rail, object corpus compatibility fixture rail, object corpus caveat taxonomy rail, and bounded local fresh-build object corpus determinism rail. It MUST preserve the existing non-proof boundaries: object evidence identifies reviewed evidence/provenance bytes and replay inputs, not behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-machine/cross-toolchain determinism.

#### Scenario: Aggregate gate composes object evidence rails

- GIVEN a maintainer invokes the aggregate Valence object evidence Nix check
- WHEN the check evaluates and builds
- THEN it depends on the object corpus replay, compatibility, caveat taxonomy, and fresh-build determinism rails
- AND it exits successfully only when all composed rails succeed

#### Scenario: Contributor docs name the aggregate command and boundaries

- GIVEN a contributor reads the contributor guide
- WHEN they look for Valence object-evidence verification
- THEN the guide names the aggregate Nix check command
- AND it states the aggregate gate is bounded evidence replay rather than a correctness, semantic-equivalence, exhaustive-coverage, or cross-machine determinism proof
