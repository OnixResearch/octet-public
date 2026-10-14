## MODIFIED Requirements

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

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST continue to point proposal-stage, delta-spec-stage, and design-stage authors at the repo-local OpenSpec template files before implementation.

`openspec/config.yaml` MUST also include stage rules that mention the repeated omission-prevention prompts for proposal, delta-spec, and design stages.

#### Scenario: Config guidance includes promoted omission-prevention checks

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal, delta-spec, and design prompts are prepared for a new change
- THEN the proposal rules mention verification/evidence expectations, exact affected artifacts, concrete scoped package/profile or fixture membership when applicable, downstream validation boundaries, and explicit non-goal boundaries
- AND the delta-spec rules mention proposal/design obligation cross-checks, boundary cases, negative controls, and exact proof output expectations
- AND the design rules mention scenario coverage, proposal/spec obligation cross-checks, boundary cases, and exact proof acceptance signals
