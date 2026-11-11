# Second Duplicate Specification

## Requirements

### Requirement: Duplicate identity from second specification
r[example.registry.duplicate] The second system MUST declare this requirement.

#### Scenario: Second declaration
- GIVEN the second specification
- WHEN registry export runs
- THEN the declaration MUST be rejected as a duplicate.
