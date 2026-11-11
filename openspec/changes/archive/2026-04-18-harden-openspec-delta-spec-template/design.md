# Design: Harden OpenSpec Delta-Spec Template

## Context

Tigerstyle already ships repo-local templates for `proposal.md`, `design.md`,
and `tasks.md`, but not for change-local delta specs. Recent review churn still
clusters around proposal/spec-stage omissions: missing explicit `The system MUST
...` requirement sentences, missing `#### Scenario:` blocks, artifact contracts
that name only a path, and confusion between delta-form change specs and main
full-form specs under `openspec/specs/`.

This change stays repo-local. It adds one copyable delta-spec template and makes
current contributor/config guidance point authors at that template before
implementation.

## Goals / Non-Goals

**Goals:**
- give maintainers one copyable delta-spec shape before they write change-local specs
- front-load repo-specific guardrails that recent review repeatedly requested
- make contributor/config guidance point at the same template path
- save deterministic proof for the exact template and guidance text

**Non-Goals:**
- generic upstream OpenSpec changes
- retroactive rewrites of archived delta specs
- adding new contributor-guide or config coverage for `openspec/templates/tasks.md`; that stays under `openspec-task-governance`
- broader task-governance or implementation-policy changes outside spec authoring

## Decisions

### 1. Add one repo-local delta-spec template at `openspec/templates/spec.md`

**Choice:** Ship `openspec/templates/spec.md` as the single copyable template
for change-local delta specs.

The template keeps this exact second-level-heading inventory, in order, with no
additional second-level headings:

- `## ADDED Requirements`
- `## MODIFIED Requirements`
- `## REMOVED Requirements`

It also keeps these exact prompt lines:

- `### Requirement: <requirement heading>`
- `The system MUST <normative behavior>`
- `#### Scenario: <scenario heading>`
- `- GIVEN <context>`
- `- WHEN <action>`
- `- THEN <observable outcome>`
- `This requirement is removed because <reason or replacement>.`

**Rationale:** The repeated spec-stage failures are mostly about missing shape,
not missing implementation effort. One copyable template removes heading and
prompt guesswork before review starts.

**Alternative considered:** Leave delta specs free-form and rely on proposal
or design review to ask for missing structure. Rejected because that keeps the
same omission churn in the most repeated authoring hotspot.

**Implementation:** Add `openspec/templates/spec.md` with the exact heading and
prompt inventory above.

### 2. Encode repo-specific guardrails directly in the template comments

**Choice:** Put tigerstyle-specific spec-authoring guardrails in exact template
comments at the top of `openspec/templates/spec.md`.

Those comments state that:
- change-local specs stay in delta form while `openspec/specs/*/spec.md` stays
  in full-spec form
- added or modified requirements start with `The system MUST ...`
- every added or modified requirement gets at least one `#### Scenario:` block
- proof-artifact requirements describe exact expected output or schema, not
  only a path

**Rationale:** These are repo-local failure patterns already captured in
`AGENTS.md` and the napkin. Putting them in the template is cheaper and more
reliable than rediscovering them in review.

**Alternative considered:** Keep the guardrails only in `AGENTS.md` and napkin
notes. Rejected because those files are not the copyable surface maintainers
use when drafting a new change.

**Implementation:** Add the exact five guidance comments promised by the delta
spec requirement.

### 3. Point contributor and proposal-stage guidance at the delta-spec template

**Choice:** Update `docs/contributor-guide.md` to add a standalone
`[OpenSpec delta-spec template](../openspec/templates/spec.md)` line, and update
`openspec/config.yaml` so proposal-stage guidance names both
`openspec/templates/proposal.md` and `openspec/templates/spec.md` before design
work begins.

This change intentionally leaves `openspec/templates/tasks.md` discovery under
existing `openspec-task-governance` guidance instead of pulling task-template
rules into `openspec-stage-authoring`.

**Rationale:** In this repo, proposal-stage work already covers proposal plus
change-local delta specs. Reusing that existing guidance entry point is lower
risk than inventing a new config stage.

**Alternative considered:** Add the template file without changing discoverable
entry points. Rejected because authors would still miss the template unless they
already knew the path.

**Implementation:** Add the contributor-guide link line and add one proposal
rule that names `openspec/templates/spec.md` alongside the existing proposal
rule for `openspec/templates/proposal.md`.

### 4. Prove shape and guidance with deterministic text extracts

**Choice:** Save deterministic extracts under
`openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/`.

Expected outputs:

- `spec-template-fields.txt` contains these exact comment lines plus the full
  extracted `## ` heading inventory and required prompt lines from
  `openspec/templates/spec.md`:
  - `<!-- Copy this file into openspec/changes/<change>/specs/<domain>/spec.md and replace the placeholders. -->`
  - `<!-- Change-local specs stay in delta form. Main specs under openspec/specs/<domain>/spec.md stay in full spec form. -->`
  - `<!-- Start each added or modified requirement with an explicit sentence beginning "The system MUST ...". -->`
  - `<!-- Give every added or modified requirement at least one "#### Scenario:" block. -->`
  - `<!-- If acceptance depends on a saved artifact, describe the artifact's exact expected output or schema in the requirement body, not only the path. -->`
  - `## ADDED Requirements`
  - `## MODIFIED Requirements`
  - `## REMOVED Requirements`
  - `### Requirement: <requirement heading>`
  - `The system MUST <normative behavior>`
  - `#### Scenario: <scenario heading>`
  - `- GIVEN <context>`
  - `- WHEN <action>`
  - `- THEN <observable outcome>`
  - `This requirement is removed because <reason or replacement>.`
- `openspec-stage-authoring-spec-crosswalk.txt` contains these exact lines:
  - `[OpenSpec proposal template](../openspec/templates/proposal.md)`
  - `[OpenSpec delta-spec template](../openspec/templates/spec.md)`
  - `[OpenSpec design template](../openspec/templates/design.md)`
  - `proposal rule -> openspec/templates/proposal.md`
  - `proposal rule -> openspec/templates/spec.md`
  - `design rule -> openspec/templates/design.md`
- `proposal-gate.txt` starts with the exact replay command
  `openspec_gate stage=proposal change=harden-openspec-delta-spec-template` and
  then captures the resulting proposal-stage review transcript.

**Rationale:** This change promises exact template and guidance text. Exact
extracts are more reviewable than screenshots or summary prose.

## Risks / Trade-offs

- More template ceremony -> keep the heading and prompt set minimal and tied to repeated failures only
- Guidance drift from template text -> save one crosswalk artifact that proves contributor/config entry points match the template paths
- Overfitting to current review pain -> keep the template small and adjust only if later review evidence justifies more fields
