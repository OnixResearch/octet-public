# Resilience Specification

## Purpose

Tiger Style resilience requirements for reviewable incident analysis: symptom-to-design reasoning, blast-radius and invariant review, retry and degradation assumptions, and explicit fail-open versus fail-closed decisions.
## Requirements
### Requirement: Public Resilience Review Guide

The project MUST ship a public resilience review guide at
`docs/resilience-review.md` that teaches maintainers to climb from recurring
symptoms and short-term workarounds to the design decision underneath them.

That guide MUST cover:

- the observed incident, symptom, and immediate workaround
- a five-whys chain that reaches a design decision rather than stopping at
  restart or retry
- boundary, blast radius, and affected invariants
- retry, replay, idempotence, and degradation assumptions
- an explicit fail-open versus fail-closed decision
- explicit reviewer prompts and follow-up ownership

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

That guide MUST keep these exact second-level headings:

- `## Incident, symptom, and immediate workaround`
- `## Five whys to the design decision`
- `## Boundary, blast radius, and affected invariants`
- `## Retry, replay, idempotence, and degradation`
- `## Fail-open vs fail-closed`
- `## Reviewer prompts and follow-ups`

#### Scenario: Guide traces a recurring incident to a design decision

- GIVEN a maintainer opens `docs/resilience-review.md`
- WHEN they read the guide
- THEN they are prompted to climb from symptom and workaround to a design
  decision instead of stopping at retry or restart

### Requirement: Copyable Five-Whys Architecture Template

The project MUST ship a copyable template at
`templates/five-whys-architecture.md`.

That template MUST include fill-in fields for:

- the incident or scenario under review
- the symptom and immediate workaround
- the five-whys chain
- boundary, blast radius, and affected invariants
- retry, replay, idempotence, and degradation assumptions
- the fail-open versus fail-closed decision
- follow-ups, owners, and non-goals

Every template section MUST stay a fill-in field or reviewer prompt rather than
free-form prose.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template.

That template MUST keep these exact second-level headings:

- `## Incident or scenario under review`
- `## Symptom and immediate workaround`
- `## Five whys chain`
- `## Boundary, blast radius, and affected invariants`
- `## Retry, replay, idempotence, and degradation assumptions`
- `## Fail-open vs fail-closed decision`
- `## Follow-ups, owners, and non-goals`

#### Scenario: Template captures a five-whys architecture review

- GIVEN a maintainer needs to capture a resilience review or postmortem
- WHEN they copy `templates/five-whys-architecture.md`
- THEN they can record the five-whys chain and surrounding resilience
  assumptions without inventing a format first

### Requirement: Human Review Record For Resilience Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`
for the published guide and template.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/resilience-review.md, templates/five-whys-architecture.md`
- `question 1: <pass|fail> - guide climbs from symptom and workaround to a design decision rather than stopping at retry or restart`
- `question 2: <pass|fail> - guide makes blast radius, affected invariants, retry or replay assumptions, and degradation posture explicit`
- `question 3: <pass|fail> - guide requires an explicit fail-open versus fail-closed decision, follow-up ownership, and the rule that restart or retry alone is not root cause`
- `question 4: <pass|fail> - every required guide heading includes at least one explicit reviewer prompt or checklist question`
- `question 5: <pass|fail> - template covers every required section and each section stays a fill-in prompt rather than free-form prose`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 5` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the guide or template is corrected.

#### Scenario: Reviewer signs off resilience guidance quality

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  paths, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 5` are all `pass`
- AND `verdict` is `pass`

