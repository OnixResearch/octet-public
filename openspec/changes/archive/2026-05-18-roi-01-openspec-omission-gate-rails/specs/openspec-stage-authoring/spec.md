## MODIFIED Requirements

### Requirement: Proposal Template Captures Scope, Artifacts, And Ordering

The system MUST continue to ship a copyable OpenSpec proposal template at
`openspec/templates/proposal.md` with the existing required heading order and
prompt lines.

The template MUST also include an omission-prevention prompt that tells authors
to name concrete verification/evidence expectations, affected artifacts, and
out-of-scope boundaries before writing specs or tasks.

#### Scenario: Maintainer sees repeated proposal omission checks before drafting

- GIVEN a maintainer copies `openspec/templates/proposal.md`
- WHEN they draft a new change proposal
- THEN the template asks them to name verification or evidence expectations
  before review
- AND the template asks them to name affected files or artifacts
- AND the template asks them to name explicit non-goals or boundaries

### Requirement: Delta-Spec Template Carries Repo-Specific Guardrails

The system MUST continue to keep the existing repo-specific guidance comments in
`openspec/templates/spec.md`.

The template MUST also include an omission-prevention prompt that tells authors
to enumerate boundary cases, negative controls, and continued-warning or
continued-blocking scenarios when those cases are part of the requirement.

#### Scenario: Maintainer sees repeated delta-spec omission checks before review

- GIVEN a maintainer opens `openspec/templates/spec.md`
- WHEN they draft an added or modified requirement
- THEN the template reminds them to include boundary cases or negative controls
  when behavior has non-trigger cases
- AND it reminds them to include continued-warning or continued-blocking
  scenarios when regressions must still be caught
- AND it keeps the existing requirement that proof artifacts describe exact
  expected output or schema rather than only a path

### Requirement: Design Template Captures Ordered Proof Planning

The system MUST continue to ship a copyable OpenSpec design template at
`openspec/templates/design.md` with the existing required heading order,
decision-block labels, and deterministic proof-plan labels.

The template MUST also include an omission-prevention prompt that tells authors
to check every added or modified scenario heading and every zero-scenario
requirement against design decisions, boundary cases, and proof acceptance
signals before implementation starts.

#### Scenario: Maintainer sees repeated design omission checks before implementation

- GIVEN a maintainer copies `openspec/templates/design.md`
- WHEN they fill the decisions and deterministic proof plan
- THEN the template reminds them to cover every added or modified scenario
  heading or zero-scenario requirement
- AND it reminds them to name boundary cases and continued-warning cases
- AND it reminds them to state exact proof acceptance signals for saved artifacts

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST continue to point proposal-stage, delta-spec-stage, and
design-stage authors at the repo-local OpenSpec template files before
implementation.

`openspec/config.yaml` MUST also include stage rules that mention the repeated
omission-prevention prompts for proposal, delta-spec, and design stages.

#### Scenario: Config guidance includes promoted omission-prevention checks

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal, delta-spec, and design prompts are prepared for a new change
- THEN the proposal rules mention verification/evidence expectations and exact
  affected artifacts
- AND the delta-spec rules mention boundary cases, negative controls, and exact
  proof output expectations
- AND the design rules mention scenario coverage, boundary cases, and exact
  proof acceptance signals
