## ADDED Requirements

### Requirement: Missing-artifact behavior
r[demo.missing_artifact] The system MUST report missing required artifacts.

#### Scenario: Missing artifact scenario
r[demo.missing_artifact.scenario]
- GIVEN a fixture change with missing required artifacts
- WHEN cairn plans sync
- THEN it reports blocker reasons
