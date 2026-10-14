## ADDED Requirements

### Requirement: Structured Task Units

Every `V<n>` task MUST verify exactly one named scenario.

#### Scenario: One scenario does not hide inside a bundled checkbox

- GIVEN a delta spec adds two scenario headings
- WHEN the maintainer writes verification tasks
- THEN one `V<n>` task names only one scenario

#### Scenario: Shared transcript still keeps separate scenario rows

- GIVEN two scenarios share one artifact
- WHEN the maintainer writes `## Verification Coverage`
- THEN both scenarios still keep separate rows
