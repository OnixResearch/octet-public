## ADDED Requirements

### Requirement: Kani pilot selection
r[octet.kani_panic_pilots.selection] Octet SHOULD select Kani panic-freedom pilot surfaces from deterministic functional cores, arithmetic or encoding boundaries, unsafe-boundary wrappers, and compact state-transition code where symbolic input coverage is valuable.

#### Scenario: Functional-core pilot is selected
r[octet.kani_panic_pilots.selection.functional_core]
- GIVEN a deterministic core function has no external I/O or runtime dependencies
- WHEN Octet selects pilot Kani coverage
- THEN the function SHOULD be eligible for a symbolic panic-freedom or property harness.

#### Scenario: Out-of-scope surface is deferred
r[octet.kani_panic_pilots.selection.deferred]
- GIVEN a candidate surface depends on unmodeled concurrency, async scheduling, or FFI behavior
- WHEN Octet selects pilot Kani coverage
- THEN the candidate SHOULD be deferred or modeled with explicit boundary assumptions.

### Requirement: Kani harness convention
r[octet.kani_panic_pilots.harness_convention] Octet Kani pilot harnesses MUST declare property class, input admission boundary, assumptions, Kani mode, replay path, and receipt linkage.

#### Scenario: Harness metadata is complete
r[octet.kani_panic_pilots.harness_convention.complete]
- GIVEN a pilot harness is added
- WHEN reviewers inspect the harness metadata
- THEN the property class, admitted inputs, assumptions, mode, replay path, and receipt linkage MUST be visible.

#### Scenario: Incomplete metadata is rejected
r[octet.kani_panic_pilots.harness_convention.incomplete]
- GIVEN a pilot harness lacks admitted-input or assumption metadata
- WHEN Octet validates pilot evidence
- THEN validation MUST fail or mark the harness as review-incomplete.

### Requirement: Assumption review boundary
r[octet.kani_panic_pilots.assumption_review] Octet MUST require reviewable justification for Kani assumptions and safety-invariant admission used by pilot harnesses.

#### Scenario: Reviewed assumption is accepted
r[octet.kani_panic_pilots.assumption_review.accepted]
- GIVEN a pilot harness uses `kani::assume` to restrict inputs to a documented validity boundary
- WHEN reviewers validate the pilot evidence
- THEN the assumption MAY be accepted as part of the scoped proof context.

#### Scenario: Vacuous assumption is rejected
r[octet.kani_panic_pilots.assumption_review.vacuous]
- GIVEN a pilot harness assumption makes the input space empty or unjustified
- WHEN Octet validates pilot evidence
- THEN the harness MUST NOT satisfy the pilot evidence requirement.

### Requirement: Positive pilot evidence
r[octet.kani_panic_pilots.positive_evidence] Octet MUST include passing Kani pilot evidence for selected panic-freedom or boundary-safety properties before documenting the pilot as usable.

#### Scenario: Passing pilot records property evidence
r[octet.kani_panic_pilots.positive_evidence.passing]
- GIVEN a selected pilot harness passes Kani verification for its declared scope
- WHEN Octet records the result
- THEN a Kani rail receipt MUST bind the passing verdict to the harness and source hash.

#### Scenario: Failed pilot remains diagnostic
r[octet.kani_panic_pilots.positive_evidence.failed]
- GIVEN a selected pilot harness fails Kani verification
- WHEN Octet records the result
- THEN the failure MUST remain diagnostic evidence and MUST NOT be promoted to passing evidence.

### Requirement: Negative pilot evidence
r[octet.kani_panic_pilots.negative_evidence] Octet pilot coverage MUST include negative fixtures or replay cases for malformed inputs, invalid assumptions, or expected failing assertions.

#### Scenario: Malformed input fails correctly
r[octet.kani_panic_pilots.negative_evidence.malformed]
- GIVEN a negative pilot fixture violates the declared input boundary
- WHEN the fixture is checked
- THEN the harness or receipt validation MUST fail with deterministic diagnostics.

#### Scenario: Expected failing assertion is replayable
r[octet.kani_panic_pilots.negative_evidence.replay]
- GIVEN Kani reports a concrete counterexample for an expected failing assertion
- WHEN the replay path is executed
- THEN the failure MUST reproduce as a standard Rust or Kani test artifact.

### Requirement: Pilot receipt linkage
r[octet.kani_panic_pilots.receipt_linkage] Octet MUST link pilot harness outputs to Kani rail receipts without treating pilot success as whole-workspace verification.

#### Scenario: Receipt linkage preserves scope
r[octet.kani_panic_pilots.receipt_linkage.scoped]
- GIVEN a pilot Kani receipt passes for a specific harness
- WHEN Octet exports or verifies pilot evidence
- THEN the receipt MUST preserve the harness scope and source hash.

#### Scenario: Whole-workspace promotion is rejected
r[octet.kani_panic_pilots.receipt_linkage.no_workspace_promotion]
- GIVEN a pilot receipt is presented as proof of all workspace behavior
- WHEN Octet validates the evidence
- THEN validation MUST reject the overbroad claim.

### Requirement: Kani pilot boundary documentation
r[octet.kani_panic_pilots.boundary_docs] Octet documentation MUST explain that panic-freedom pilots are targeted evidence and that stronger functional claims require contracts, passing unwind coverage, or separate proof rails.

#### Scenario: Pilot guide states non-claims
r[octet.kani_panic_pilots.boundary_docs.visible]
- GIVEN a consumer reads the Kani pilot guide
- WHEN pilot evidence is described
- THEN the guide MUST state the targeted scope and non-claims.
