## MODIFIED Requirements

### Requirement: Structured Task Units

Every active tigerstyle `tasks.md` file MUST include exact task obligations for scenario-specific verification.

#### Scenario: Task obligations map scenarios to exact proof

- GIVEN a delta spec scenario requires an omitted-field default, explicit override, required artifact, negative control, or continued-warning behavior
- WHEN the maintainer writes a `V<n>` verification task for that scenario
- THEN the task names the exact scenario, command or helper, input or fixture, saved evidence path, and expected acceptance signal
- AND the task cannot be satisfied by a broader check that omits that scenario-specific behavior

#### Scenario: Task obligation fixtures prove blocked and covered tasks

- GIVEN the task-obligation fixture pair
- WHEN the runner executes each fixture
- THEN missing task obligations are blocked and covered task obligations pass
