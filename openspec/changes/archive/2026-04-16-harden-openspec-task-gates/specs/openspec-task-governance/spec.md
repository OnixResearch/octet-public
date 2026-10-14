## ADDED Requirements

### Requirement: Structured Task Units

Every active tigerstyle `tasks.md` file for a new or modified capability MUST
use structured checkbox task IDs and keep implementation, verification, and
human/oracle claims in separate checkbox units.

For this capability, "active" means a change directory directly under
`openspec/changes/` and explicitly excludes `openspec/changes/archive/**`.

The supported task-ID classes are:

- `I<n>` for implementation work
- `V<n>` for deterministic verification work
- `H<n>` for human/oracle checkpoints

Those IDs MUST be unique within the file.

`I<n>` tasks MUST describe one implementation claim and MUST NOT also claim
verification or evidence capture in the same checkbox.

`V<n>` tasks MUST verify exactly one named scenario or one requirement-level
invariant that has no scenario heading.

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

### Requirement: Verification Coverage Matrix

Every active tigerstyle `tasks.md` file for a change with delta specs MUST end
with a `## Verification Coverage` section.

That section MUST contain one mapping row for every added or modified scenario
heading, plus one row for any requirement-level invariant that has no scenario
heading.

A `Requirement:` row is only valid when that requirement has zero scenario
headings in the parsed delta spec. If a requirement has one or more scenarios,
those scenario rows are mandatory and a requirement row MUST NOT replace them.

Each mapping row MUST use this exact shape:

- `` `Scenario: <heading>` -> `V<n>|H<n>` -> `<repo-relative evidence path>` ``
- `` `Requirement: <heading>` -> `V<n>|H<n>` -> `<repo-relative evidence path>` ``

A scenario MAY share the same evidence artifact as another scenario, but it MUST
still keep its own mapping row.

#### Scenario: Reviewer can trace a scenario to proof

- GIVEN a reviewer reads a delta spec scenario
- WHEN they open `tasks.md`
- THEN they can find one exact coverage row for that scenario
- AND that row names the task ID and saved evidence path without guesswork

#### Scenario: Shared transcript still keeps separate scenario rows

- GIVEN two scenarios are proven by the same saved UI transcript
- WHEN the maintainer writes the coverage section
- THEN both scenarios still appear as separate rows
- AND both rows may point at the same evidence path

### Requirement: Human/Oracle Checkpoints Name Exact Inputs

Every `H<n>` checkbox MUST use this exact field layout in the task text:

`Human/oracle checkpoint: command=`<command or helper>`; input=`<pinned revision, fixture, or prerequisite artifact>`; asserts=`<single human claim>`; evidence=`<repo-relative evidence path>`

The `command=` field MAY point at a helper script instead of an inline shell
command, but it MUST still identify the exact replay path.

#### Scenario: Aspen replay task is self-describing

- GIVEN a maintainer needs a downstream replay against a pinned Aspen commit
- WHEN they read the `H<n>` task
- THEN the task itself names the replay command or helper
- AND it names the pinned Aspen revision or prerequisite artifact
- AND it names the saved evidence file used for review

### Requirement: Tasks-Stage Gate Flags Missing Governance

Tigerstyle's task template and tasks-stage gate MUST reject task files that omit
this governance contract.

At minimum, the tasks-stage gate MUST report blocking findings for these cases:

- an `I<n>` checkbox mixes implementation with verification or evidence-capture
  claims; the blocking rule is structural and triggers when an `I<n>` task text
  includes verification/evidence markers such as `verify`, `save`, `capture`,
  `evidence=`, or a repo-relative `evidence/` path
- a delta-spec scenario or requirement-level invariant has no coverage row
- an `H<n>` checkbox omits any of `command=`, `input=`, `asserts=`, or
  `evidence=`

#### Scenario: Bundled implementation task is blocked

- GIVEN `tasks.md` contains `- [ ] I3 update parser and save cargo-test-lib.txt`
- WHEN the tasks-stage gate runs
- THEN it reports that `I3` mixes implementation and verification claims

#### Scenario: Missing scenario mapping is blocked

- GIVEN a delta spec adds `Scenario: Bare disabled key still beats source-level attrs`
- AND `tasks.md` has no matching coverage row
- WHEN the tasks-stage gate runs
- THEN it reports that the scenario has no verification coverage

#### Scenario: Human checkpoint missing evidence path is blocked

- GIVEN `tasks.md` contains `H2 Human/oracle checkpoint: command=...; input=...; asserts=...`
- WHEN the tasks-stage gate runs
- THEN it reports that `H2` is missing the required `evidence=` field
