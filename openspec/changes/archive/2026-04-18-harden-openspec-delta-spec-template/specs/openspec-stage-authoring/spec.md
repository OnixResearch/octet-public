## ADDED Requirements

### Requirement: Delta-Spec Template Captures Normative Delta Shape

The system MUST ship a copyable OpenSpec delta-spec template at
`openspec/templates/spec.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings:

- `## ADDED Requirements`
- `## MODIFIED Requirements`
- `## REMOVED Requirements`

That template MUST also keep these exact prompt lines:

- `### Requirement: <requirement heading>`
- `The system MUST <normative behavior>`
- `#### Scenario: <scenario heading>`
- `- GIVEN <context>`
- `- WHEN <action>`
- `- THEN <observable outcome>`
- `This requirement is removed because <reason or replacement>.`

#### Scenario: Maintainer drafts a delta spec without inventing headings

- GIVEN a maintainer copies `openspec/templates/spec.md`
- WHEN they draft a change-local delta spec under `openspec/changes/<change>/specs/<domain>/spec.md`
- THEN they can add or modify requirements without inventing extra second-level headings
- AND they can start each added or modified requirement from an explicit `The system MUST ...` sentence
- AND they can add a named `#### Scenario:` block with `GIVEN` / `WHEN` / `THEN` steps

### Requirement: Delta-Spec Template Carries Repo-Specific Guardrails

The delta-spec template at `openspec/templates/spec.md` MUST keep these exact
repo-specific guidance comments:

- `<!-- Copy this file into openspec/changes/<change>/specs/<domain>/spec.md and replace the placeholders. -->`
- `<!-- Change-local specs stay in delta form. Main specs under openspec/specs/<domain>/spec.md stay in full spec form. -->`
- `<!-- Start each added or modified requirement with an explicit sentence beginning "The system MUST ...". -->`
- `<!-- Give every added or modified requirement at least one "#### Scenario:" block. -->`
- `<!-- If acceptance depends on a saved artifact, describe the artifact's exact expected output or schema in the requirement body, not only the path. -->`

#### Scenario: Maintainer sees repo-specific delta-spec guardrails in the template

- GIVEN a maintainer opens `openspec/templates/spec.md`
- WHEN they read the template comments before drafting a delta spec
- THEN they can see that active-change specs stay in delta form while main specs stay in full-spec form
- AND they can see that added or modified requirements begin with `The system MUST ...`
- AND they can see that each added or modified requirement needs at least one `#### Scenario:` block
- AND they can see that proof-artifact requirements must describe exact expected output or schema, not just a path

## MODIFIED Requirements

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST point proposal-stage, delta-spec-stage, and design-stage
authors at the repo-local OpenSpec template files before implementation.

`docs/contributor-guide.md` MUST contain these exact standalone markdown-link
lines:

- `[OpenSpec proposal template](../openspec/templates/proposal.md)`
- `[OpenSpec delta-spec template](../openspec/templates/spec.md)`
- `[OpenSpec design template](../openspec/templates/design.md)`

`openspec/config.yaml` MUST contain one proposal rule that names
`openspec/templates/proposal.md` and one proposal rule that names
`openspec/templates/spec.md` for delta-spec authoring, plus one design rule
that names `openspec/templates/design.md`.

#### Scenario: Contributor guide exposes the delta-spec template

- GIVEN a maintainer opens `docs/contributor-guide.md`
- WHEN they look for repo-local OpenSpec authoring guidance
- THEN they can find the standalone delta-spec-template link line above
- AND the same guide section still exposes the proposal-template and design-template link lines above

#### Scenario: Proposal-stage guidance names the delta-spec template

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal-stage prompts are prepared for a new change
- THEN one proposal rule names `openspec/templates/proposal.md`
- AND one proposal rule names `openspec/templates/spec.md`
- AND design guidance still names `openspec/templates/design.md`
