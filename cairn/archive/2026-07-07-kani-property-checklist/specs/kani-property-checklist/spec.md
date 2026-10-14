## ADDED Requirements

### Requirement: Kani property classes
r[octet.kani_property_checklist.property_classes] Octet SHOULD provide a Kani/property checklist covering normalization, algebraic laws, encode/decode identity, validity ranges, loop termination, trait consistency, and boundary panic behavior.

#### Scenario: Checklist class is available to reviewers
r[octet.kani_property_checklist.property_classes.available]
- GIVEN a reviewer is evaluating Kani or property-test evidence
- WHEN the reviewer opens the checklist
- THEN the supported property classes SHOULD be visible with examples.

#### Scenario: Non-applicable class can be skipped
r[octet.kani_property_checklist.property_classes.skip]
- GIVEN a property class does not apply to a module
- WHEN reviewers fill out the checklist
- THEN the class MAY be marked not applicable with a short rationale.

### Requirement: Kani receipt property labels
r[octet.kani_property_checklist.receipt_labels] Octet MAY record checklist property-class labels in Kani receipts, and validation MUST fail closed or preserve the label as review-only metadata when the label is unknown.

#### Scenario: Known property label is preserved
r[octet.kani_property_checklist.receipt_labels.known]
- GIVEN a Kani receipt names a supported checklist property class
- WHEN Octet validates the receipt
- THEN the property label MUST be preserved in deterministic receipt metadata.

#### Scenario: Unknown property label is not promoted
r[octet.kani_property_checklist.receipt_labels.unknown]
- GIVEN a Kani receipt names an unsupported property class
- WHEN Octet validates evidence tiers
- THEN the label MUST NOT promote the receipt beyond its verified result.

### Requirement: Checklist selection guidance
r[octet.kani_property_checklist.selection_guidance] Octet documentation MUST explain how reviewers select relevant property classes for a module and how they document skipped classes.

#### Scenario: Reviewer selects applicable classes
r[octet.kani_property_checklist.selection_guidance.select]
- GIVEN a module exposes arithmetic, encoding, or trait behavior
- WHEN reviewers apply the checklist
- THEN relevant property classes MUST be selected or explicitly marked not applicable.

#### Scenario: Skipped class is reviewable
r[octet.kani_property_checklist.selection_guidance.skip]
- GIVEN reviewers skip a checklist class
- WHEN the evidence is reviewed
- THEN the rationale MUST be visible and reviewable.

### Requirement: Positive and negative checklist coverage
r[octet.kani_property_checklist.positive_negative] Each adopted checklist property SHOULD include positive evidence for expected behavior and negative evidence for invalid, malformed, stale, overflow, or trait-law failure cases.

#### Scenario: Positive evidence demonstrates expected behavior
r[octet.kani_property_checklist.positive_negative.positive]
- GIVEN a module adopts a checklist property
- WHEN the evidence suite runs
- THEN at least one passing check SHOULD demonstrate the expected behavior.

#### Scenario: Negative evidence demonstrates failure behavior
r[octet.kani_property_checklist.positive_negative.negative]
- GIVEN a module adopts a checklist property
- WHEN invalid or malformed fixture inputs are checked
- THEN the suite SHOULD show deterministic rejection or counterexample behavior.

### Requirement: Trait consistency checklist
r[octet.kani_property_checklist.trait_consistency] Octet SHOULD include explicit guidance for checking `Eq`, `Ord`, and `Hash` consistency where Rust trait implementations define semantic contracts.

#### Scenario: Equality and ordering agree
r[octet.kani_property_checklist.trait_consistency.eq_ord]
- GIVEN a type implements equality and ordering traits
- WHEN reviewers adopt the trait consistency checklist item
- THEN evidence SHOULD check that equality implies equal ordering where the trait contract requires it.

#### Scenario: Inconsistent traits produce negative evidence
r[octet.kani_property_checklist.trait_consistency.negative]
- GIVEN a fixture violates an adopted trait consistency property
- WHEN the evidence suite runs
- THEN the fixture SHOULD fail with deterministic diagnostics or a concrete counterexample.

### Requirement: Property-evidence graduation guidance
r[octet.kani_property_checklist.graduation] Octet documentation MUST explain how panic-freedom evidence can graduate to contract-backed Kani evidence or separate formal proof rails when stronger functional claims are required.

#### Scenario: Stronger claim names stronger evidence
r[octet.kani_property_checklist.graduation.stronger]
- GIVEN a requirement needs functional correctness beyond panic-freedom
- WHEN the checklist guidance is followed
- THEN the guidance MUST direct reviewers to contracts, passing unwind coverage, or formal proof rails.
