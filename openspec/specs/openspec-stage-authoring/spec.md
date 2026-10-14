# OpenSpec Stage Authoring Specification

## Purpose

Define tigerstyle's repo-local OpenSpec proposal-stage, delta-spec-stage,
and design-stage authoring contract: copyable proposal, delta-spec, and
design templates with exact heading inventories, explicit repo-relative
artifact planning, and contributor/config guidance that points maintainers at
those templates before implementation.
## Requirements
### Requirement: Proposal Template Captures Scope, Artifacts, And Ordering

The system MUST continue to ship a copyable OpenSpec proposal template at `openspec/templates/proposal.md` with the existing required heading order and prompt lines.

The template MUST also include omission-prevention prompts that tell authors to name concrete verification/evidence expectations, exact affected artifacts, concrete package/profile or other scoped membership when applicable, downstream validation boundaries, and explicit non-goal boundaries before writing specs, design, or tasks.

#### Scenario: Maintainer sees repeated proposal omission checks before drafting

- GIVEN a maintainer copies `openspec/templates/proposal.md`
- WHEN they draft a new change proposal
- THEN the template asks them to name verification or evidence expectations before review
- AND the template asks them to name affected files or artifacts
- AND the template asks them to name explicit non-goals or boundaries

#### Scenario: Proposal author records exact scope and downstream validation boundaries

- GIVEN a proposal affects a concrete package set, profile set, fixture set, or downstream validation target
- WHEN the proposal is prepared for gate review
- THEN the proposal names that concrete scope rather than only a broad category
- AND it names downstream validation boundaries and saved evidence expectations that later specs, design, and tasks can carry forward

#### Scenario: Proposal governance fixtures prove blocked and covered obligations

- GIVEN the fixture `tests/fixtures/openspec-proposal-governance/proposal-obligation-missing-blocked`
- AND the fixture `tests/fixtures/openspec-proposal-governance/proposal-obligation-covered-pass`
- WHEN `tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh` runs each fixture through the proposal gate
- THEN the missing fixture is accepted only when the gate blocks the omitted proposal obligation
- AND the covered fixture is accepted only when the gate passes the explicit proposal obligations
- AND saved evidence for this change is written only under this change's evidence directory and eventual archive copy, not by editing historical archives

### Requirement: Design Template Captures Ordered Proof Planning

The system MUST continue to ship a copyable OpenSpec design template at
`openspec/templates/design.md` with the existing required heading order,
decision-block labels, deterministic proof-plan labels, and an explicit
obligation cross-check prompt.

The template MUST also include omission-prevention prompts that tell authors to
check every added or modified scenario heading, every zero-scenario requirement,
every proposal verification/evidence expectation, every boundary or
negative-control obligation, and every planned saved proof artifact against
design decisions and proof acceptance signals before implementation starts.

#### Scenario: Maintainer sees repeated design omission checks before implementation

- GIVEN a maintainer copies `openspec/templates/design.md`
- WHEN they fill the decisions and deterministic proof plan
- THEN the template reminds them to cover every added or modified scenario
  heading or zero-scenario requirement
- AND it reminds them to name boundary cases and continued-warning cases
- AND it reminds them to map proposal promises and delta-spec obligations into
  design decisions or proof-plan entries
- AND it reminds them to state exact proof acceptance signals for saved artifacts

#### Scenario: Design author cross-checks proposal and spec obligations

- GIVEN a proposal names verification expectations, affected artifacts, or
  non-goal boundaries
- AND a delta spec names scenarios, boundary cases, negative controls, or
  continued-warning expectations
- WHEN the design is prepared for gate review
- THEN the design includes an obligation cross-check that maps each such
  proposal or spec obligation to a decision, boundary case, or deterministic
  proof-plan entry

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

The system MUST continue to keep the existing repo-specific guidance comments in
`openspec/templates/spec.md`.

The template MUST also include omission-prevention prompts that tell authors to
enumerate boundary cases, negative controls, and continued-warning or
continued-blocking scenarios when those cases are part of the requirement.

The template MUST include an obligation cross-check prompt that tells authors to
map every proposal verification/evidence promise, affected artifact, non-goal
boundary, and design decision obligation into an explicit delta-spec requirement
or named scenario before proposal-stage gate review.

#### Scenario: Maintainer sees repeated delta-spec omission checks before review

- GIVEN a maintainer opens `openspec/templates/spec.md`
- WHEN they draft an added or modified requirement
- THEN the template reminds them to include boundary cases or negative controls
  when behavior has non-trigger cases
- AND it reminds them to include continued-warning or continued-blocking
  scenarios when regressions must still be caught
- AND it keeps the existing requirement that proof artifacts describe exact
  expected output or schema rather than only a path

#### Scenario: Spec author cross-checks proposal and design obligations

- GIVEN a proposal names verification expectations, affected artifacts, or
  non-goal boundaries
- AND a design names decisions, proof obligations, boundary cases, negative
  controls, or continued-warning expectations
- WHEN a delta spec is prepared for gate review
- THEN the delta spec includes requirements or named scenarios for each such
  proposal or design obligation

#### Scenario: Spec governance fixtures prove blocked and covered obligations

- GIVEN the fixture `tests/fixtures/openspec-spec-governance/spec-obligation-missing-blocked`
- AND the fixture `tests/fixtures/openspec-spec-governance/spec-obligation-covered-pass`
- WHEN `tests/fixtures/openspec-spec-governance/run-spec-fixture-gate.sh` runs each fixture through the proposal gate
- THEN the missing fixture is accepted only when the gate blocks the omitted proposal obligation
- AND the covered fixture is accepted only when the gate passes the named obligation scenario
- AND saved evidence for this change is written only under this change's evidence directory and eventual archive copy, not by editing historical archives

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST continue to point proposal-stage, delta-spec-stage, and design-stage authors at the repo-local OpenSpec template files before implementation.

`openspec/config.yaml` MUST also include stage rules that mention the repeated omission-prevention prompts for proposal, delta-spec, and design stages.

#### Scenario: Config guidance includes promoted omission-prevention checks

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal, delta-spec, and design prompts are prepared for a new change
- THEN the proposal rules mention verification/evidence expectations, exact affected artifacts, concrete scoped package/profile or fixture membership when applicable, downstream validation boundaries, and explicit non-goal boundaries
- AND the delta-spec rules mention proposal/design obligation cross-checks, boundary cases, negative controls, and exact proof output expectations
- AND the design rules mention scenario coverage, proposal/spec obligation cross-checks, boundary cases, and exact proof acceptance signals
