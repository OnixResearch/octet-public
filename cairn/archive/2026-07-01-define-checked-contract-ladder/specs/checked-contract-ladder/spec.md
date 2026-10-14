## ADDED Requirements

### Requirement: Progressive assurance ladder

r[octet.checked_contract_ladder.stages] Octet documentation SHOULD define a progressive assurance ladder that maps scoped safety or correctness obligations to explicit evidence tiers, commands or gates, and artifacts or receipts.

#### Scenario: Ladder names every stage

r[octet.checked_contract_ladder.stages.inventory]

- GIVEN a reader opens the checked-contract or trust-boundary documentation
- WHEN they inspect the progressive assurance ladder
- THEN the documentation SHOULD name advisory lint findings, warning gates, deny gates, policy requirements, receipt-backed validation, scoped formal-proof receipts, and verified release artifacts as distinct stages.
- AND each stage SHOULD name its evidence tier or state that the stage is operational gating rather than formal proof.

#### Scenario: Ladder stage points to review surface

r[octet.checked_contract_ladder.stages.review_surface]

- GIVEN a ladder stage is documented
- WHEN a reviewer asks how to inspect that stage
- THEN the stage SHOULD point to the relevant command, gate, artifact, receipt, or provenance surface when one exists.

### Requirement: Checked-contract records

r[octet.checked_contract_ladder.contract_records] Octet SHOULD define a checked-contract record as a scoped requirement, invariant, or review obligation tied to evidence; each record SHOULD include requirement or policy ID, source scope, invariant or failure shape, evidence tier, command or gate, artifact or receipt path, reviewer action, and explicit non-claim.

#### Scenario: Checked-contract fields are defined

r[octet.checked_contract_ladder.contract_records.fields]

- GIVEN documentation introduces checked contracts
- WHEN it defines the checked-contract record
- THEN it SHOULD define the requirement or policy ID field.
- AND it SHOULD define the source scope field.
- AND it SHOULD define the invariant or failure-shape field.
- AND it SHOULD define the evidence-tier field.
- AND it SHOULD define the command or gate field.
- AND it SHOULD define the artifact or receipt path field.
- AND it SHOULD define the reviewer-action field.
- AND it SHOULD define the explicit non-claim field.

#### Scenario: Checked contract maps to existing surfaces

r[octet.checked_contract_ladder.contract_records.surface_mapping]

- GIVEN a checked-contract example appears in Octet documentation
- WHEN the example shows how evidence is produced
- THEN it SHOULD map the record to existing lint, standards policy, proof-card, provenance, or artifact-verifier surfaces.
- AND it SHOULD NOT require a new Rust syntax extension to be valid.

### Requirement: Deterministic failure-shape diagnostics

r[octet.checked_contract_ladder.diagnostic_shapes] High-confidence Octet diagnostic examples SHOULD name the invalid state, branch, missing guard, resource boundary, or remediation path when that information is available from deterministic analysis.

#### Scenario: Diagnostic example names failure shape

r[octet.checked_contract_ladder.diagnostic_shapes.example]

- GIVEN documentation shows a high-confidence Octet lint or policy finding
- WHEN the analysis can identify a deterministic failure shape
- THEN the example SHOULD name the invalid state, branch, missing guard, resource boundary, or remediation path.
- AND it SHOULD avoid generic wording that leaves the reviewer unable to tell what condition failed.

#### Scenario: Diagnostic example avoids SMT-counterexample claim

r[octet.checked_contract_ladder.diagnostic_shapes.no_smt_overclaim]

- GIVEN a diagnostic example does not come from an SMT or Verus counterexample
- WHEN documentation describes the example
- THEN it MUST NOT claim that Octet produced a solver model or formal counterexample.

### Requirement: Checked-contract boundaries

r[octet.checked_contract_ladder.boundaries] Checked-contract documentation MUST preserve Octet's proof-vs-validation boundaries and MUST NOT imply Salt-style language syntax, general solver completeness, runtime fallback checks, whole-program proof, or that every ladder stage is formal proof.

#### Scenario: Unsupported checked-contract claim fails validation

r[octet.checked_contract_ladder.boundaries.overclaim_check]

- GIVEN documentation validation scans checked-contract and assurance-ladder docs
- WHEN it encounters unsupported wording that Octet has built-in `requires` or `ensures` syntax, complete solver-backed verification for arbitrary Rust, runtime fallback checks for unproved contracts, whole-program proof, or formal proof at every ladder stage
- THEN validation MUST fail with a diagnostic naming the unsupported claim.

#### Scenario: Formal proof remains scoped

r[octet.checked_contract_ladder.boundaries.scoped_proof]

- GIVEN the assurance ladder includes scoped formal-proof receipts
- WHEN the documentation explains that stage
- THEN it MUST scope the proof to the declared property, source/proof binding, verifier identity, receipt, and artifact identity.
- AND it MUST keep lint, policy, transcript, validation, and manual-review evidence in their own evidence tiers.
