## MODIFIED Requirements

### Requirement: Structured Task Units

Every active tigerstyle `tasks.md` file for a new or modified capability MUST use structured checkbox task IDs and keep implementation, verification, and human/oracle claims in separate checkbox units.

For this capability, "active" means a change directory directly under `openspec/changes/` and explicitly excludes `openspec/changes/archive/**`.

The supported task-ID classes are:

- `I<n>` for implementation work
- `V<n>` for deterministic verification work
- `H<n>` for human/oracle checkpoints

Those IDs MUST be unique within the file.

`I<n>` tasks MUST describe one implementation claim and MUST NOT also claim verification or evidence capture in the same checkbox.

`V<n>` tasks MUST verify exactly one named scenario or one requirement-level invariant that has no scenario heading, and each `V<n>` task MUST name the exact command/helper, required input or artifact, evidence path, and acceptance signal needed to prove that scenario.

`H<n>` tasks MUST confirm exactly one human/oracle claim.

#### Scenario: Implementation and verification are split

- GIVEN a change needs code edits and a saved test transcript
- WHEN the maintainer writes `tasks.md`
- THEN the code edit lives in an `I<n>` checkbox
- AND the saved transcript lives in a separate `V<n>` or `H<n>` checkbox

#### Scenario: One scenario does not hide inside a bundled checkbox

- GIVEN a delta spec adds a scenario named `Explicit warn override still warns`
- WHEN the maintainer adds verification tasks
- THEN one `V<n>` task names that scenario directly
- AND the same checkbox does not also claim unrelated scenario coverage

#### Scenario: Task obligations map scenarios to exact proof

- GIVEN a delta spec scenario requires an omitted-field default, explicit override, required artifact, negative control, or continued-warning behavior
- WHEN the maintainer writes a `V<n>` verification task for that scenario
- THEN the task names the exact scenario, command or helper, input or fixture, saved evidence path, and expected acceptance signal
- AND the task cannot be satisfied by a broader check that omits that scenario-specific behavior

### Requirement: Task Template Shows Change-Status Fields

Tigerstyle's repo-local task template at `openspec/templates/tasks.md` MUST continue to include a `## Change Status` section immediately before `## Verification Coverage` with the existing required example lines and note.

The template MUST also include omission-prevention prompts that tell authors to split implementation from verification, give every scenario its own coverage row, make `H<n>` oracle tasks self-verifiable with exact `command=`, `input=`, `asserts=`, and `evidence=` fields, and make each `V<n>` task name the exact scenario, command/helper, required artifact or fixture, evidence path, and acceptance signal.

#### Scenario: Maintainer sees repeated task omission checks before gate review

- GIVEN a maintainer copies `openspec/templates/tasks.md`
- WHEN they write implementation, verification, and human/oracle tasks
- THEN the template reminds them not to combine implementation with evidence capture in a single `I<n>` task
- AND it reminds them that every added or modified scenario needs its own verification coverage row
- AND it reminds them that each `H<n>` task needs exact replay fields

#### Scenario: Task obligations map scenarios to exact proof

- GIVEN a delta spec scenario requires an omitted-field default, explicit override, required artifact, negative control, or continued-warning behavior
- WHEN the maintainer writes `V<n>` tasks from the template
- THEN the template tells them to name that exact scenario, command/helper, input or fixture, evidence path, and expected PASS output or diagnostic
- AND it warns that broad tasks such as `test config parsing` are insufficient when the scenario requires a specific absent-field default, explicit override, or continued-warning case

### Requirement: Tasks-Stage Gate Flags Missing Governance

The system MUST make tigerstyle's tasks-stage gate report the existing canonical `openspec-task-governance` blocking findings plus repeated task-obligation omissions where a required scenario, artifact, exact acceptance signal, negative control, or continued-warning case is not represented by an explicit `V<n>`/coverage mapping.

#### Scenario: Task obligation fixtures prove blocked and covered tasks

- GIVEN the fixture `tests/fixtures/openspec-task-governance/task-obligation-missing-blocked`
- AND the fixture `tests/fixtures/openspec-task-governance/task-obligation-covered-pass`
- WHEN `tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh` runs each fixture through the tasks gate
- THEN the missing fixture is accepted only when the gate blocks the omitted task obligation
- AND the covered fixture is accepted only when the gate passes explicit task/evidence mappings
- AND saved evidence for this change is written only under this change's evidence directory and eventual archive copy, not by editing historical archives

### Requirement: Tasks-Stage Config Guidance Names Omission Checks

`openspec/config.yaml` MUST include tasks-stage rules that mention the repeated omission-prevention prompts for split implementation/proof tasks, one-row-per-scenario coverage, exact per-scenario verification acceptance signals, required artifacts/fixtures, and exact human/oracle replay fields.

#### Scenario: Config task guidance includes promoted task omission checks

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN tasks-stage prompts are prepared for a new change
- THEN the tasks rules mention split implementation, verification, and human/oracle tasks
- AND the tasks rules mention one verification coverage row per scenario
- AND the tasks rules mention exact `V<n>` scenario commands, required artifacts or fixtures, evidence paths, and acceptance signals
- AND the tasks rules mention exact `H<n>` `command=`, `input=`, `asserts=`, and `evidence=` fields
