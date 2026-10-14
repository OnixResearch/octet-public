## MODIFIED Requirements

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
- AND the delta-spec rules mention proposal/design obligation cross-checks,
  boundary cases, negative controls, and exact proof output expectations
- AND the design rules mention scenario coverage, proposal/spec obligation
  cross-checks, boundary cases, and exact proof acceptance signals
