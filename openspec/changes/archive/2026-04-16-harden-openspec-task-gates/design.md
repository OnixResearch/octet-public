## Context

Tigerstyle's current tasks-stage review pain is repetitive and well-localized.
Repo-local `review_metrics promotions` currently shows the highest repeated
finding count in `omission / tasks / prompt` (32), followed by
`omission / tasks / human` (21) and `incoherent / tasks / human` (12).
The repeated examples are also consistent:

- one checkbox tries to cover implementation plus proof capture
- delta-spec scenarios are present but tasks do not expose an auditable mapping
  to verification work
- human/oracle tasks rely on tribal knowledge instead of naming the exact
  replay command, pinned input, and saved evidence path

The current repo rules in `openspec/config.yaml` say tasks must cover every
MUST and record validation evidence, but they do not give a parseable shape for
how that must appear in `tasks.md`. This change adds a repo-local contract that
later implementation can enforce in both task authoring templates and the
`openspec_gate` tasks stage.

Scope is limited to active changes under `openspec/changes/<change>/` and
explicitly excludes `openspec/changes/archive/**`.

## Goals / Non-Goals

**Goals:**
- make task files auditable before implementation is marked done
- make spec-scenario-to-proof coverage obvious from `tasks.md`
- make human/oracle checkpoints replayable without consulting napkin history
- give the tasks-stage gate deterministic failure classes that match the review
  churn already measured in `review_metrics`

**Non-Goals:**
- upstream generic OpenSpec schema changes outside tigerstyle's repo-local
  workflow
- retroactive cleanup of archived task files except where a fixture needs a
  real past failure mode
- proposal-stage or design-stage hardening beyond task-governance needs

## Decisions

### 1. Task checkboxes use explicit `I`, `V`, and `H` IDs

**Choice:** Every checkbox starts with a unique task ID in one of three
classes:

- `I<n>`: one implementation claim
- `V<n>`: one deterministic verification claim for exactly one scenario or one
  requirement-level invariant with no scenario heading
- `H<n>`: one human/oracle claim

`I<n>` tasks must not also claim verification or saved evidence. `V<n>` and
`H<n>` tasks each cover one proof claim only.

**Rationale:** The review pain is not that maintainers forget to write tasks at
all. The pain is that task text is too free-form to audit. Typed IDs let the
repo distinguish implementation, deterministic proof, and human checkpoints
without guessing from prose alone.

**Alternative considered:** Keep free-form checklist prose and try to infer task
intent with looser heuristics. Rejected because the recent failures already show
that free-form prose leaves too much room for bundled claims and reviewer
reinterpretation.

**Implementation:** The future template will show phase sections for
implementation, deterministic verification, and human/oracle checkpoints. The
future tasks-stage gate will parse the task ID prefix and enforce the class-
specific rules from the spec. For `V<n>` tasks, the unit is one scenario
heading, not one GIVEN/WHEN/THEN bullet chain; multiple GIVEN bullets under the
same scenario heading still count as one verification target.

### 2. Scenario coverage lives in an explicit matrix, not only prose tasks

**Choice:** `tasks.md` must end with `## Verification Coverage` rows in this
exact shape:

- `` `Scenario: <heading>` -> `V<n>|H<n>` -> `<repo-relative evidence path>` ``
- `` `Requirement: <heading>` -> `V<n>|H<n>` -> `<repo-relative evidence path>` ``

Every added or modified scenario gets its own row, even when multiple rows point
at the same evidence artifact.

**Rationale:** The highest repeated omission findings came from scenarios that
existed in the delta spec but were only vaguely implied in tasks. A dedicated
matrix makes coverage reviewable without forcing the gate to parse natural-
language descriptions or infer which scenario a generic "run UI tests" task was
supposed to prove.

**Alternative considered:** Require each `V<n>` checkbox to embed the full
scenario heading and evidence path, with no separate matrix. Rejected because
maintainers still need a single place where reviewers can audit total scenario
coverage across the whole change.

**Implementation:** The gate will read the change delta specs, enumerate added
and modified scenario headings, then confirm a one-row-per-heading mapping in
`tasks.md`. Requirement-level rows are only valid when a requirement has zero
scenario headings after parsing; they are not a substitute for missing scenario
rows.

### 3. Human/oracle tasks use fixed replay fields

**Choice:** `H<n>` tasks must use this exact field layout:

`Human/oracle checkpoint: command=`<command or helper>`; input=`<pinned revision, fixture, or prerequisite artifact>`; asserts=`<single human claim>`; evidence=`<repo-relative evidence path>`

**Rationale:** The repeated human-route findings were not about whether human
validation should exist; they were about the task text leaving out the replay
path, pinned input, or saved artifact. Fixed fields make those omissions easy to
spot.

**Alternative considered:** Keep free-form human task prose but add a style note
in the template. Rejected because review history already shows that style notes
alone do not stop ambiguous checkpoint tasks.

**Implementation:** The gate will parse the four required fields. Helper-script
commands are allowed, but the field must still name the exact helper path so the
saved transcript can be reproduced.

### 4. Blocking failures match the measured churn classes

**Choice:** The hardened tasks-stage gate will produce blocking findings for
three deterministic failures:

- `Task I<n> mixes implementation and verification/evidence claims`
- `Scenario "<heading>" has no verification coverage row`
- `Human/oracle task H<n> is missing command=, input=, asserts=, or evidence=`

For the mixed-claim failure, the blocking rule is structural: an `I<n>` task is
invalid if its text contains verification/evidence markers such as `verify`,
`save`, `capture`, `evidence=`, or a repo-relative `evidence/` path.

**Rationale:** These are the same failure shapes dominating repo-local review
metrics. Matching the emitted findings to those classes will make the follow-on
implementation measurable: if the change works, those counts should fall first.

**Alternative considered:** Treat all likely bundling as warnings only. Rejected
for the specific mixed implementation+proof case because that exact pattern kept
causing false-done task states.

**Implementation:** The gate may keep non-deterministic prose quality feedback
as warnings, but the three listed failures must block the tasks stage.

### 5. Proof uses small fixture changes drawn from real failure modes

**Choice:** Verification will use small repo-local fixture changes that exercise
one failure at a time, plus one passing example that follows the new template.
The fixture set will cover:

- bundled `I<n>` task that also claims saved evidence
- missing scenario coverage row
- malformed `H<n>` task missing one required field
- passing `tasks.md` with typed IDs, full coverage matrix, and explicit replay
  fields

**Rationale:** The implementation needs deterministic proof, not a hand-waved
claim that future authors will follow the template.

**Implementation:** The follow-on change can place those fixtures under
`tests/fixtures/openspec-task-governance/` or an equivalent repo-local test
area, then save task-gate transcripts under the active change `evidence/`
directory.

## Risks / Trade-offs

**[Task authoring overhead]** -> Typed IDs and coverage rows add ceremony.
Mitigation: keep the syntax short, provide a copyable template, and allow shared
evidence artifacts across multiple scenario rows.

**[Gate false positives on old free-form tasks]** -> Existing archived changes
will not match the new contract. Mitigation: scope enforcement to active changes
that opt into the new governance capability, and use dedicated fixtures instead
of retroactive archive cleanup.

**[Human checkpoint sprawl]** -> Authors may create too many `H<n>` tasks when a
single deterministic `V<n>` task would do. Mitigation: keep `H<n>` reserved for
claims that truly need a human/oracle judgment or an external replay boundary.
