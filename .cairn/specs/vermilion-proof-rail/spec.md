# Vermilion Proof Rail Specification

## Purpose

Defines the `vermilion-proof-rail` capability.

## Requirements

### Requirement: Pinned Vermilion provider profile
r[octet.vermilion_proof_rail.provider_profile] Octet MUST define a closed Nickel provider profile that binds the Vermilion revision, Verus fork and vstd revisions, translator revision and argv, Lean toolchain, Mathlib and dependency-manifest identities, checker argv, environment allowlist, resource limits, allowed axioms, proof-hole markers, and non-claims.

#### Scenario: Complete profile is accepted
- GIVEN the reviewed profile supplies every required pin and identity
- WHEN Nickel export and pure profile validation run
- THEN the generated profile MUST be deterministic and preserve every pin.
- AND its BLAKE3 identity MUST match the embedded profile identity.

#### Scenario: Provider pin drifts
- GIVEN a profile changes the Verus fork, vstd, translator, Lean, Mathlib, dependency audit, checker, environment, or axiom policy
- WHEN profile validation runs
- THEN validation MUST fail before any provider process could execute.

### Requirement: Common semantic contract and refinement bridges
r[octet.vermilion_proof_rail.semantic_contract] Octet MUST bind one common bounded-increment statement with width, overflow behavior, precondition, postcondition, observable failure, termination scope, and one obligation-and-bridge declaration for each Verus, Vermilion, and Aeneas route.

#### Scenario: Routes align with the common contract
- GIVEN a route declares the same statement identity, width, overflow behavior, precondition, and result type
- WHEN comparison eligibility is evaluated
- THEN the route may participate only after its refinement bridge is checked.

#### Scenario: A nat claim substitutes for u64
- GIVEN the Verus route claims a `nat` monotonicity property or another arithmetic width
- WHEN comparison eligibility is evaluated
- THEN the route MUST be incomparable and the missing refinement bridge MUST remain an open obligation.

### Requirement: Typed receipt bindings
r[octet.vermilion_proof_rail.receipt] Octet MUST bind a Vermilion receipt to exact provider, semantic-spec, source, toolchain, dependency-audit, command, environment, limit, generated-artifact, proof-twin, obligation, theorem, axiom, checker, outcome, evidence-role, comparison, and non-claim identities.

#### Scenario: Complete receipt is accepted
- GIVEN every binding matches the reviewed profile and contract
- WHEN receipt validation and evaluation run with an available closure
- THEN the derived outcome MUST be `supported`.
- AND a synthetic fixture MUST remain `recorded_only`.

#### Scenario: Stale generated output or proof twin is supplied
- GIVEN a generated artifact, proof twin, source byte, or receipt projection changed after measurement
- WHEN receipt validation runs
- THEN validation MUST fail with the artifact, twin, source, or tamper diagnostic.
- AND no property evidence MUST be admitted.

### Requirement: Every obligation is accounted for
r[octet.vermilion_proof_rail.obligations] Octet MUST require the receipt obligation set to equal the reviewed contract set by identity and kind, and MUST require every obligation to be checked with no omitted obligation, empty checker output, unresolved proof hole, or axiom outside the reviewed allowance.

#### Scenario: Omitted obligation or unexpected axiom is present
- GIVEN an obligation is missing, extra, unresolved, rejected, or depends on an unapproved axiom
- WHEN receipt evaluation runs
- THEN the outcome MUST be `failed`.
- AND the primary diagnostic MUST identify the obligation or axiom class.

#### Scenario: Checker cannot report a complete result
- GIVEN the checker is absent, interrupted, exited non-zero, or emitted no machine-readable result
- WHEN receipt evaluation runs
- THEN the outcome MUST be `blocked` for an absent closure and `failed` otherwise.
- AND it MUST NOT be `supported`.

### Requirement: Closed comparison vocabulary
r[octet.vermilion_proof_rail.comparison] Octet MUST classify cross-provider evidence as `comparable_agreement`, `semantic_mismatch`, `incomparable_claims`, `proof_failure`, `unsupported_or_blocked`, `timeout_or_exhaustion`, or `tool_error`, and MUST map each category explicitly onto the existing Aeneas and Vermilion outcome spellings without broadening those enums.

#### Scenario: All routes check the same property
- GIVEN every route checked the same statement, semantics, and refinement bridge
- WHEN comparison runs
- THEN the category MUST be `comparable_agreement`.
- AND evidence authority MUST NOT merge.

#### Scenario: Providers disagree or cannot execute
- GIVEN routes differ in semantics, claims, proof result, support, budget, or tool execution
- WHEN comparison runs
- THEN the category MUST distinguish the mismatch, failure, blocker, timeout, or tool error.
- AND only categories the existing outcome vocabulary can express MUST map to `supported`, `blocked`, or `failed`.

### Requirement: Negative controls are exercised
r[octet.vermilion_proof_rail.negative_controls] Octet MUST include positive and negative fixtures for source substitution, spec substitution, weakened preconditions, changed overflow semantics, `nat`/`u64` confusion, wrong provider, stale generated output, stale proof twins, missing and unexpected obligations, unexpected axioms, unresolved holes, interrupted or empty checking, missing checkers, missing dependency audits, tampering, non-claim drift, evidence overclaim, and absent toolchains.

#### Scenario: Negative matrix runs
- GIVEN the positive contract fixture and the declared mutation matrix
- WHEN the focused fixture tests run
- THEN every case MUST produce its declared outcome and primary diagnostic class.
- AND no case may admit property evidence.

### Requirement: Trust boundaries and blockers are explicit
r[octet.vermilion_proof_rail.trust_boundary] Octet documentation MUST retain a human-readable claim and trust matrix, distinguish implemented contract surfaces from designed downstream integration, and report unavailable external toolchains as blockers rather than passes.

#### Scenario: External closure is absent
- GIVEN Vermilion, Lean, Lake, or cvc5 is not available
- WHEN evaluation runs against the probe
- THEN the outcome MUST be `blocked` with the missing-checker class.
- AND the documentation MUST state that no live proof or comparison was measured.

#### Scenario: Synthetic fixture requests property evidence
- GIVEN a synthetic contract fixture sets the property evidence role
- WHEN receipt evaluation runs
- THEN the outcome MUST be `failed` with `evidence_overclaim`.
