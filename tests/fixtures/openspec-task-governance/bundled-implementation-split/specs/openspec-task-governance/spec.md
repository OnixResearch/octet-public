## ADDED Requirements

### Requirement: Structured Task Units

The tasks-stage gate MUST reject `I<n>` checkboxes that mix implementation and
saved-evidence capture in the same checkbox.

#### Scenario: Implementation and verification are split

- GIVEN `tasks.md` contains `- [ ] I1 update parser and save openspec/changes/example/evidence/parser.txt`
- WHEN the tasks-stage gate runs
- THEN it reports that `I1` mixes implementation and verification/evidence claims
