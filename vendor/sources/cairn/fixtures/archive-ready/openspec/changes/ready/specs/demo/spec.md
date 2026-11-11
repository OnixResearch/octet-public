## ADDED Requirements

### Requirement: Ready behavior
r[demo.ready] The system MUST parse archive-ready fixture delta specs.

#### Scenario: Ready scenario
r[demo.ready.scenario]
- GIVEN a completed fixture change
- WHEN cairn plans archive
- THEN it reports an archive action
