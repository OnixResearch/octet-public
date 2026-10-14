## ADDED Requirements

### Requirement: Verification Coverage Matrix

Every added or modified scenario heading MUST appear in `## Verification
Coverage`.

#### Scenario: Reviewer can trace a scenario to proof

- GIVEN a reviewer reads a delta spec scenario
- WHEN they open `tasks.md`
- THEN they can find one exact coverage row for that scenario

#### Scenario: Shared transcript still keeps separate scenario rows

- GIVEN two scenarios share one artifact
- WHEN the maintainer writes `## Verification Coverage`
- THEN each scenario still keeps its own row
