## MODIFIED Requirements

### Requirement: Aggregate Valence Object Evidence Gate [r[aggregate-valence-object-evidence-gate]]

The system MUST provide a named Nix check that aggregates Octet's selected Valence object-evidence rails for operator and CI use.

The aggregate gate MUST compose the checked-in internal object corpus replay rail, object corpus compatibility fixture rail, object corpus caveat taxonomy rail, bounded local fresh-build object corpus determinism rail, and THIR supplement fixture inventory rail. It MUST preserve the existing non-proof boundaries: object evidence identifies reviewed evidence/provenance bytes and replay inputs, not behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-machine/cross-toolchain determinism. The THIR inventory rail MUST remain selected-fixture-only and MUST NOT imply general THIR extraction.

#### Scenario: Aggregate gate composes object evidence rails

- GIVEN a maintainer invokes the aggregate Valence object evidence Nix check
- WHEN the check evaluates and builds
- THEN it depends on the object corpus replay, compatibility, caveat taxonomy, fresh-build determinism, and THIR supplement fixture inventory rails
- AND it exits successfully only when all composed rails succeed

#### Scenario: Aggregate gate composes THIR inventory rail

- GIVEN the focused THIR supplement inventory rail checks the selected THIR fixture inventory
- WHEN the aggregate Valence object evidence check builds
- THEN it depends on that THIR inventory rail
- AND its output reports `valence_object_evidence.thir_supplement_inventory_gate: PASS`

#### Scenario: Contributor docs name the aggregate command and boundaries

- GIVEN a contributor reads the contributor guide
- WHEN they look for Valence object-evidence verification
- THEN the guide names the aggregate Nix check command
- AND it states the aggregate gate is bounded evidence replay rather than a correctness, semantic-equivalence, exhaustive-coverage, or cross-machine determinism proof
- AND it states the aggregate includes selected THIR fixture inventory coverage without implying general THIR extraction
