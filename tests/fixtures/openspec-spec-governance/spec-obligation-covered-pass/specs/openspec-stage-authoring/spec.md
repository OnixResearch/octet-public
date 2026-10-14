## MODIFIED Requirements

### Requirement: Delta-Spec Template Carries Repo-Specific Guardrails

The system MUST add a spec authoring prompt and preserve proposal-named boundaries by encoding them as explicit requirements or scenarios.

#### Scenario: Maintainer sees prompt

- GIVEN a maintainer opens the template
- WHEN they draft a spec
- THEN they see the prompt

#### Scenario: Archive non-goal and evidence promise are specified

- GIVEN a proposal states that historical archived examples are not normalized
- AND the proposal requires saved evidence that archive-local references remain untouched
- WHEN the delta spec is prepared for proposal-stage gate review
- THEN the delta spec names the archive non-goal boundary
- AND it requires evidence with the exact acceptance signal `PASS archive untouched`
