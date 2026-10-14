# Valence Evidence Index Generator Specification

## Purpose

Defines the `valence-evidence-index-generator` capability for Octet/Valence evidence closeout: deterministic generation and staleness checking of the current-head Valence evidence index while preserving bounded selected-fixture claims.

## Requirements

### Requirement: The system
r[octet.valence_evidence.index_generator.render] The system MUST render `docs/current-head-valence-evidence.md` deterministically from a verified payload commit, an ordered Valence evidence rail inventory, observed pass signals, and explicit non-claim boundaries.

#### Scenario: Render Current Evidence Page
r[octet.valence_evidence.index_generator.render.matches_current]
- GIVEN the current verified payload commit and current Valence object evidence rail inventory
- WHEN the operator runs the evidence index generator
- THEN it produces Markdown that matches the committed current-head Valence evidence page except for deliberately supplied capture metadata

#### Scenario: Render Bounded Claims
r[octet.valence_evidence.index_generator.render.bounded_claims]
- GIVEN the generator renders the scope and non-claim sections
- WHEN the operator reviews the generated page
- THEN it states the evidence is bounded local selected-corpus / selected-fixture replay and does not claim correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction support

### Requirement: The system
r[octet.valence_evidence.index_generator.check] The system MUST provide a fail-closed check mode that compares generated evidence-index output with the checked-in Markdown page.

#### Scenario: Check Fresh Page
r[octet.valence_evidence.index_generator.check.fresh]
- GIVEN the checked-in current-head Valence evidence page matches the generated output for the audited payload commit
- WHEN the operator runs the evidence index check
- THEN the check exits successfully and emits a deterministic pass receipt or pass signal

#### Scenario: Check Stale Page
r[octet.valence_evidence.index_generator.check.fail_closed]
- GIVEN the checked-in page records the wrong payload commit, omits an expected rail/pass signal, or changes required non-claim text
- WHEN the operator runs the evidence index check
- THEN the check fails closed with deterministic mismatch issue codes and without rewriting the page implicitly

### Requirement: The system
r[octet.valence_evidence.index_generator.negative] The system MUST include negative fixture coverage for evidence-index drift.

#### Scenario: Negative Drift Fixtures
r[octet.valence_evidence.index_generator.negative.fixtures]
- GIVEN stale payload, missing rail, missing pass signal, and weakened non-claim fixture pages
- WHEN the check evaluates each fixture
- THEN each fixture is rejected with a stable mismatch class that identifies the drift category

### Requirement: The system
r[octet.valence_evidence.index_generator.nix_gate] The system MUST expose evidence-index freshness through a named local Nix check that can compose with Valence evidence closeout.

#### Scenario: Nix Gate Passes
r[octet.valence_evidence.index_generator.nix_gate.aggregate]
- GIVEN the checked-in evidence page is fresh for the audited payload and all required rail markers are present
- WHEN the operator runs the named evidence-index Nix check and the aggregate Valence object evidence check
- THEN both checks pass and the aggregate closeout path includes the evidence-index freshness signal
