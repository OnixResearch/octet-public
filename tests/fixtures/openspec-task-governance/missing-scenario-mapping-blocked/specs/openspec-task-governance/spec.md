## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

Every added or modified scenario heading MUST appear in `## Verification
Coverage`.

#### Scenario: Missing scenario mapping is blocked

- GIVEN a delta spec scenario has no coverage row
- WHEN the tasks-stage gate runs
- THEN it reports that the scenario has no verification coverage row

#### Scenario: Shared transcript still keeps separate scenario rows

- GIVEN two scenarios share one artifact
- WHEN the maintainer writes `## Verification Coverage`
- THEN each scenario still keeps its own row
