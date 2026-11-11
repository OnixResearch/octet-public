## ADDED Requirements

### Requirement: Proposed fixture detail
r[fixture.lifecycle_view.proposed] The system MUST preserve proposed lifecycle facts in the view.

#### Scenario: Proposed detail renders
r[fixture.lifecycle_view.proposed.render]
- GIVEN an active delta spec
- WHEN Cairn builds a lifecycle view
- THEN the change contains the proposed spec
