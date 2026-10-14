# Charon Semantic Analysis Specification

## Purpose

Defines the `charon-semantic-analysis` capability.

## Requirements

### Requirement: Charon semantic analysis is an opt-in typed profile

r[octet.charon_semantic_analysis.profile] Octet MUST define a typed Nickel `charon-semantic` evidence-rail profile that binds the reviewed provider dependency, exact source/item selection, supported semantic fact vocabulary, limits, reference metadata, artifact roles, and non-claims without changing starter, strict-profile, source-token closure, or formal-proof defaults.

#### Scenario: Complete profile passes

- GIVEN a profile names exact provider/source/item identities, a closed fact vocabulary, named limits, roles, references, and non-claims
- WHEN profile validation and export run
- THEN the generated runtime profile MUST be deterministic and the rail MAY be selected explicitly.

#### Scenario: Broad or incomplete profile fails

- GIVEN a profile uses ambient tools, branch pins, broad item matching, unknown fact kinds, missing limits, stale reference metadata, or weakened non-claims
- WHEN validation runs
- THEN validation MUST fail before extraction.

### Requirement: Semantic analysis reuses the reviewed extraction contract

r[octet.charon_semantic_analysis.extraction_contract] The rail MUST reuse the Charon package and effective extraction contract admitted by the Aeneas provider, including exact pin/options/target/selection/serialization identity, `has_errors = false`, and `skip_borrowck = false`, and MUST NOT create an independent fallback runner.

#### Scenario: Reviewed extraction is accepted

- GIVEN the provider identity, effective serialized options, target metadata, selected items, and measured artifact match the reviewed profile
- WHEN rail preflight runs
- THEN the snapshot MAY proceed to pure semantic validation.

#### Scenario: Partial or weakened extraction is rejected

- GIVEN extraction reports errors, skipped borrow checking, target/profile drift, hidden Cargo metadata, an unreviewed option, or a different selected item set
- WHEN preflight runs
- THEN the rail MUST fail closed without semantic evidence promotion.

### Requirement: Semantic decisions live in a pure core

r[octet.charon_semantic_analysis.pure_core] Octet MUST validate and normalize already-loaded Charon snapshot values through pure deterministic functions over typed source/item/span/call/dependency/trait/control-flow/opacity/unsupported/target inputs.

#### Scenario: Valid snapshot normalizes deterministically

- GIVEN an in-memory snapshot contains well-formed exact selected-item facts
- WHEN the pure projection core runs twice
- THEN both normalized projections and issue ordering MUST match.

#### Scenario: Core requires ambient authority

- GIVEN semantic validation is invoked
- WHEN the implementation boundary is inspected or tested
- THEN the core MUST NOT read files or environment, spawn processes, access clocks or networks, evaluate Nickel, or depend on mutable global state.

### Requirement: Initial semantic analyses are bounded and fail closed

r[octet.charon_semantic_analysis.bounded_analysis] The first rail profile MUST limit analysis to exact selected item facts, resolved local call/dependency/trait rows, control-flow shape, cycles, source spans, opacity/unsupported markers, and a closed conservative effect-candidate mapping, with missing or unsupported information reported as unknown or blocked.

#### Scenario: Supported exact-item facts produce analysis

- GIVEN every required selected-item fact is present and uses the supported vocabulary
- WHEN bounded analysis runs
- THEN Octet MUST emit deterministic nodes, edges, candidates, and explicit unknown counts.

#### Scenario: Unsupported fact would require guessing

- GIVEN an item is opaque, unresolved, unsupported, malformed, or would require name-based inference
- WHEN analysis runs
- THEN the rail MUST report unknown or blocked and MUST NOT infer effect absence or a resolved edge.

### Requirement: Source-token comparison preserves both authorities

r[octet.charon_semantic_analysis.comparison] Octet MAY compare a Charon semantic projection with exact function-address and source-token closure parents, but the comparison MUST preserve both parent identities and use explicit agree, disagree, Charon-only, source-token-only, or unknown outcomes without rewriting or promoting either parent.

#### Scenario: Comparable facts agree

- GIVEN both parent artifacts bind the same durable source/item identity and report compatible facts
- WHEN comparison runs
- THEN the report MAY record agreement with both parent identities.

#### Scenario: Parents differ or cannot align

- GIVEN facts disagree, source identities differ, spans are stale, or one side lacks a comparable fact
- WHEN comparison runs
- THEN the report MUST preserve disagreement or uncertainty and MUST NOT claim semantic equivalence.

### Requirement: Charon semantic execution shell is confined and bounded

r[octet.charon_semantic_analysis.shell] The rail shell MUST invoke only pinned provider argv, confine declared local artifact roots, apply named execution/resource/log limits, measure bytes before pure validation, and deny ambient executable resolution, network access, and undeclared environment inputs.

#### Scenario: Pinned execution produces typed observations

- GIVEN the reviewed provider package, profile, source selection, roots, environment allowlist, and limits match
- WHEN the shell runs the rail
- THEN it MUST return bounded typed process and artifact observations to the pure core.

#### Scenario: Shell policy is violated

- GIVEN a process cannot spawn, resolves an ambient executable, requests network access, exceeds a named limit, escapes a declared root, or consumes an undeclared environment input
- WHEN execution runs
- THEN the rail MUST fail closed with a stage-specific diagnostic and no semantic evidence promotion.

### Requirement: Charon semantic evidence is BLAKE3-bound and recorded-only

r[octet.charon_semantic_analysis.evidence] A rail run MUST emit a versioned role-separated BLAKE3 receipt binding provider/extraction profile, target, exact source/items, raw snapshot, normalized projection, analysis profile, optional comparison parents, diagnostics, limits, status, and non-claims, and artifact verification MUST remeasure local artifacts without mutation.

#### Scenario: Untampered recorded analysis passes

- GIVEN the rail completes and every declared artifact matches measured bytes
- WHEN artifact verification runs
- THEN Octet MUST accept the recorded semantic-analysis receipt and deterministic report.

#### Scenario: Receipt requests stronger evidence

- GIVEN a receipt or report requests property, formal-proof, complete-closure, effect-absence, or release-ready authority
- WHEN validation runs
- THEN validation MUST fail.

### Requirement: Upstream analysis and semantics repositories remain references

r[octet.charon_semantic_analysis.reference_boundary] Exact reviewed `mechanized-llbc` and `charon-rudra` revisions MAY inform semantic review and fixture design, but MUST remain recorded-only references and MUST NOT become runtime dependencies or transfer semantic, proof, analysis-correctness, or implementation-conformance claims.

#### Scenario: Exact reference metadata is preserved

- GIVEN a profile records exact revision, license, sentinel, and intended reference use
- WHEN validation runs
- THEN the metadata MAY appear in the receipt as non-property provenance.

#### Scenario: Reference result is promoted

- GIVEN a report treats a reference theorem or old analysis implementation as proof of the current rail or pinned Charon
- WHEN claim validation runs
- THEN validation MUST fail.

### Requirement: Stack ownership boundaries remain explicit

r[octet.charon_semantic_analysis.boundaries] Octet MUST own Charon execution and semantic analysis; Valence MAY validate exported identity/linkage/role/non-claim evidence; Kamacite MAY carry LLBC or projections as target/provenance artifacts; neither downstream layer may become a compiler IR or semantic-analysis authority through this rail.

#### Scenario: Bounded handoff is rendered

- GIVEN an Octet semantic receipt verifies
- WHEN a downstream adapter input is emitted
- THEN it MUST preserve recorded-only roles, exact identities, and ownership/non-claim boundaries.

#### Scenario: Downstream handoff claims compiler semantics

- GIVEN a handoff claims Valence or Kamacite proves Rust semantics, Charon correctness, or complete closure
- WHEN boundary validation runs
- THEN validation MUST fail.

### Requirement: Semantic rail validation covers positive and negative paths

r[octet.charon_semantic_analysis.final_validation] Before archive, the change MUST include exact-item positive canaries and negative fixtures for profile, extraction, graph/span/fact validation, unsupported/opaque inputs, comparison disagreement, process/resource failure, receipt tampering, reference promotion, default-path independence, and overclaiming output.

#### Scenario: Focused matrix is complete

- GIVEN implementation and fixtures are ready for review
- WHEN focused tests, policy freshness, artifact verification, provider upgrade canaries, default-path checks, and Cairn validation/gates run
- THEN every positive fixture MUST pass, every negative fixture MUST fail for its declared reason, and existing starter/source-token/proof behavior MUST remain unchanged.
