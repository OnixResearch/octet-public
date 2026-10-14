# Proof Artifact Library Specification Delta

## ADDED Requirements

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

### Requirement: Transform boundaries are explicit

r[octet.proof_artifact_library.bounded_transforms.boundaries] Bounded evidence transforms MUST NOT be described as unbounded tactics, complete proof search, solver completeness, semantic equivalence, whole-program correctness, certification, business-domain validation, full safety, or verified-runtime proof.

#### Scenario: Transform reports preserve non-claims

r[octet.proof_artifact_library.bounded_transforms.boundaries.no_overclaim]

- GIVEN a transform report, documentation page, proof card, or provenance summary mentions tactics, simplification, proof, verification, correctness, safety, certification, or semantic equivalence
- WHEN claim-boundary validation runs
- THEN the wording MUST scope the transform to its declared input contract, rule set, budget, output artifact, evidence tier, and non-claims.
