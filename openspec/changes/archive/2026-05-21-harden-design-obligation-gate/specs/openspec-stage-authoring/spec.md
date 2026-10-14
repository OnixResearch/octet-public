## MODIFIED Requirements

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
- AND the design rules mention scenario coverage, proposal/spec obligation
  cross-checks, boundary cases, and exact proof acceptance signals
