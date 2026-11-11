# Lifecycle View Fixture

## Requirements

### Requirement: Deterministic fixture view
r[fixture.lifecycle_view] The system MUST expose deterministic lifecycle views for the fixture.

#### Scenario: Fixture renders
r[fixture.lifecycle_view.render]
- GIVEN a valid fixture repository
- WHEN Cairn builds a lifecycle view
- THEN the resulting receipt is stable
