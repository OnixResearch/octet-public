## ADDED Requirements

### Requirement: Internal corpus caveat taxonomy report

Octet MUST provide a deterministic object corpus caveat taxonomy report for selected Rust source inputs that explains per-source-bucket dependency, effect, macro, and pure-cache blockers while preserving evidence-only boundaries.

#### Scenario: Expanded internal taxonomy fixture is deterministic
- GIVEN the checked-in expanded internal object corpus source taxonomy
- WHEN Octet generates the object corpus caveat taxonomy report as JSON
- THEN the generated report bytes match the checked-in caveat taxonomy fixture.

#### Scenario: Caveat taxonomy names per-source blockers
- GIVEN selected internal corpus source paths with dependency, effect, macro, or pure-cache blockers
- WHEN Octet emits the caveat taxonomy report
- THEN each source bucket reports object count, caveat counts, pure-cache blocked count, and blocker names deterministically.

#### Scenario: Caveat taxonomy preserves non-proof boundaries
- GIVEN an object corpus caveat taxonomy report
- WHEN boundary fields are inspected
- THEN `claims_correctness` and `claims_semantic_equivalence` remain false and the report does not claim pure-cache eligibility or semantic equivalence.
