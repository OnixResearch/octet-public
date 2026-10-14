# Proof Artifact Library Specification

## Purpose

Defines the `proof-artifact-library` capability.

## Requirements

### Requirement: Shared proof and artifact identity core

r[octet.proof_artifact_library.shared_identity] Octet SHOULD centralize deterministic proof and artifact identity helpers, including BLAKE3 content identity and canonical closure graph normalization, in shared pure code.

#### Scenario: Same content has same identity

r[octet.proof_artifact_library.shared_identity.same_content]

- GIVEN runner and standards paths compute identity for the same artifact text or bytes
- WHEN they call the shared identity core
- THEN both paths SHOULD produce the same BLAKE3 identity string.

#### Scenario: Malformed closure graph fails deterministically

r[octet.proof_artifact_library.shared_identity.bad_graph]

- GIVEN a closure graph input is malformed or missing required canonical fields
- WHEN canonicalization runs
- THEN the shared core SHOULD return a deterministic diagnostic instead of accepting inconsistent identity.

### Requirement: Shared proof receipt binding decisions

r[octet.proof_artifact_library.proof_binding] Proof rail relevance, proof receipt source binding, proof artifact binding, and verifier identity checks SHOULD be implemented once in deterministic shared code and reused by command shells.

#### Scenario: Relevant proof rail is selected consistently

r[octet.proof_artifact_library.proof_binding.relevance]

- GIVEN the same policy registry and source files are evaluated by standards and runner code
- WHEN proof rail relevance is computed
- THEN both paths SHOULD select the same relevant proof rails.

#### Scenario: Receipt binding is stable

r[octet.proof_artifact_library.proof_binding.stable]

- GIVEN a proof receipt binds source, proof artifact, verifier, and closure identities
- WHEN binding validation runs repeatedly
- THEN the shared core SHOULD produce stable pass or fail diagnostics for equivalent inputs.

### Requirement: Fail-closed equivalence across command surfaces

r[octet.proof_artifact_library.fail_closed_equivalence] Refactors that deduplicate proof and artifact logic MUST preserve fail-closed behavior for stale hashes, missing bindings, unsupported schemas, malformed closure material, and missing provenance.

#### Scenario: Valid evidence still passes

r[octet.proof_artifact_library.fail_closed_equivalence.valid]

- GIVEN proof and artifact evidence satisfied the previous contract
- WHEN the shared core is used by runner and standards paths
- THEN the evidence MUST still pass without changing the accepted wire shape.

#### Scenario: Invalid evidence still fails

r[octet.proof_artifact_library.fail_closed_equivalence.invalid]

- GIVEN proof or artifact evidence has stale hashes, missing bindings, unsupported schemas, malformed closure material, or missing provenance
- WHEN validation runs through either command surface
- THEN validation MUST fail with deterministic diagnostics naming the rejected field or artifact.

### Requirement: Bounded evidence transforms

r[octet.proof_artifact_library.bounded_transforms] Octet SHOULD model tactic-like evidence derivation as named, bounded, deterministic transforms with explicit input contracts, output contracts, rule sets, budgets, diagnostics, and non-claims.

#### Scenario: Valid transform output is deterministic

r[octet.proof_artifact_library.bounded_transforms.deterministic]

- GIVEN a transform receives valid typed inputs, supported schema versions, declared parent artifacts, and a supported rule set
- WHEN the transform runs
- THEN it SHOULD emit deterministic output that names the transform ID, input identities, output identity, rule-set identity, evidence tier, diagnostics, and non-claims.
- AND repeated runs with equivalent inputs SHOULD produce byte-stable machine-readable output.

#### Scenario: Invalid transform fails closed

r[octet.proof_artifact_library.bounded_transforms.fail_closed]

- GIVEN a transform receives malformed input, unsupported schema versions, unknown rules, stale parent artifacts, invalid evidence-tier promotion, missing non-claims, or input that exceeds its named budget
- WHEN the transform runs
- THEN it MUST fail closed with deterministic diagnostics naming the rejected input, rule, artifact, tier, or budget.
- AND it MUST NOT emit derived evidence that can support promotion-grade or `formal_proof` provenance.

### Requirement: Rewrite and normalization sets are explicit

r[octet.proof_artifact_library.rewrite_sets] Octet SHOULD use explicit, named rewrite or normalization sets for policy, receipt, proof-card, proof-ref, artifact-identity, and claim-boundary transforms rather than hidden global simplification.

#### Scenario: Applied rules are visible

r[octet.proof_artifact_library.rewrite_sets.explicit]

- GIVEN a transform normalizes policy, receipt, proof-card, proof-ref, artifact identity, or claim-boundary material
- WHEN it reports derived output
- THEN the output SHOULD name the rewrite or normalization set identity and the transform family.
- AND unsupported, hidden, global, or undeclared rule invocation MUST fail closed.

### Requirement: Transform provenance links parents

r[octet.proof_artifact_library.transform_provenance] Derived evidence artifacts SHOULD record transform identity and parent artifact identity in provenance before they can support release or proof review.

#### Scenario: Derived artifact records parent provenance

r[octet.proof_artifact_library.transform_provenance.parents]

- GIVEN a transform emits a derived artifact that summarizes, normalizes, links, or gates earlier evidence
- WHEN provenance is emitted or verified
- THEN the provenance SHOULD name the transform ID, rule-set identity, parent artifact identities, output artifact identity, checker ID when applicable, evidence tier, checked scope, and non-claims.
- AND missing, stale, or cyclic parent provenance MUST fail closed with deterministic diagnostics.

### Requirement: Trellis proof IR intake is recorded-only by default

r[octet.proof_artifact_library.trellis_proof_ir_intake.recorded_only] Octet MUST treat imported Trellis proof evidence IR as recorded-only reference evidence unless an Octet-owned proof rerun receipt satisfies promotion checks.

#### Scenario: Reference proof does not count as Octet proof
r[octet.proof_artifact_library.trellis_proof_ir_intake.positive]
- GIVEN Octet imports a valid Trellis proof evidence-chain envelope
- WHEN no Octet-owned verifier rerun receipt is provided
- THEN Octet MUST record the artifact as reference evidence and MUST NOT count it as a promoted formal proof.

### Requirement: Intake validates source, IR, verifier, and assumptions

r[octet.proof_artifact_library.trellis_proof_ir_intake.contract] Octet MUST validate Trellis proof evidence identity shape, source envelope linkage, proof IR fields, verifier receipt metadata, assumptions, property IDs, and non-claim boundaries.

#### Scenario: Stale or incomplete proof evidence is rejected
r[octet.proof_artifact_library.trellis_proof_ir_intake.negative]
- GIVEN Trellis proof evidence has stale source, missing verifier identity, missing assumptions, malformed property IDs, or weakened non-claims
- WHEN Octet intake validation runs
- THEN Octet MUST reject or mark the evidence invalid with deterministic diagnostics.

### Requirement: Formal-proof promotion requires Octet rerun identity

r[octet.proof_artifact_library.trellis_proof_ir_intake.promotion] Octet MUST require matching source, verifier/toolchain identity, solver identity, command identity, assumptions, property scope, and passed result status before promoting Trellis-derived proof evidence to Octet formal-proof role.

#### Scenario: Unauthorized role promotion fails closed
- GIVEN imported Trellis evidence declares formal-proof status without an Octet rerun receipt
- WHEN Octet validates the intake record
- THEN validation MUST fail with an unauthorized-promotion diagnostic.

### Requirement: Assumption compatibility is explicit

r[octet.proof_artifact_library.trellis_proof_ir_intake.assumptions] Octet MUST compare Trellis proof assumptions and trusted boundaries against Octet policy before any promotion.

#### Scenario: Stack smoke preserves the proof boundary
r[octet.proof_artifact_library.trellis_proof_ir_intake.validation]
- GIVEN Trellis proof IR flows through the stack smoke
- WHEN Octet consumes it as reference evidence
- THEN the evidence chain MUST preserve recorded-only boundaries unless Octet reruns the proof.

### Requirement: Transform boundaries are explicit

r[octet.proof_artifact_library.bounded_transforms.boundaries] Bounded evidence transforms MUST NOT be described as unbounded tactics, complete proof search, solver completeness, semantic equivalence, whole-program correctness, certification, business-domain validation, full safety, or verified-runtime proof.

#### Scenario: Transform reports preserve non-claims

r[octet.proof_artifact_library.bounded_transforms.boundaries.no_overclaim]

- GIVEN a transform report, documentation page, proof card, or provenance summary mentions tactics, simplification, proof, verification, correctness, safety, certification, or semantic equivalence
- WHEN claim-boundary validation runs
- THEN the wording MUST scope the transform to its declared input contract, rule set, budget, output artifact, evidence tier, and non-claims.
