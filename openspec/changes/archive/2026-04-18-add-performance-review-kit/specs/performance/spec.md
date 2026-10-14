## ADDED Requirements

### Requirement: Public Tiger Style Performance Review Guide

The project MUST ship a public performance review guide at
`docs/performance-review.md` that translates Tiger Style performance principles
into a repeatable review checklist.

That guide MUST cover:

- early performance design before profiling
- why these checklist sections matter instead of presenting them as slogans
- napkin math
- network, disk, memory, and CPU reasoning with both bandwidth and latency
- resource priority and cost × frequency ranking
- control plane vs data plane separation
- batching, limits, queues, and backpressure
- CPU predictability, large work chunks, and reduced dependence on compiler
  optimization luck
- when to move from sketches to measurement or profiling
- explicit guidance that static lints and benchmarks do not replace workload
  reasoning

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide. This
keeps deterministic verification strong enough to prove the public surface did
not drift.

That guide MUST keep these exact second-level headings:

- `## Workload and hot path`
- `## Napkin math`
- `## Resource sketch: network, disk, memory, CPU`
- `## Cost × frequency ranking`
- `## Control plane vs data plane`
- `## Batching, limits, queues, and backpressure`
- `## CPU predictability and compiler luck`
- `## Measurement and validation`

#### Scenario: Reviewing a hot path

- GIVEN a maintainer is reviewing a hot path or runtime-sensitive API
- WHEN they follow `docs/performance-review.md`
- THEN the guide prompts them to sketch latency, bandwidth, batching, and
  predictability before micro-optimizing

#### Scenario: Auditing queue or buffer growth

- GIVEN a design introduces a queue, buffer, channel, or collector loop
- WHEN the maintainer uses `docs/performance-review.md`
- THEN the guide prompts explicit bounds, batching, and cost × frequency
  reasoning instead of only code-style cleanup

### Requirement: Copyable Performance Sketch Template

The project MUST ship a copyable performance-sketch template at
`templates/performance-sketch.md`.

That template MUST include fill-in fields for:

- workload or entry point
- assumptions and units
- rough bandwidth and latency math
- resource budgets or limits per relevant resource
- cost × frequency ranking
- control-plane vs data-plane split
- batching, limits, queues, and backpressure
- measurement or validation plan
- open risks and non-goals

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template. This
keeps deterministic verification strong enough to prove every required field
remains present.

That template MUST keep these exact second-level headings:

- `## Workload or entry point`
- `## Assumptions and units`
- `## Bandwidth and latency sketch`
- `## Resource budgets or limits`
- `## Cost × frequency ranking`
- `## Control-plane vs data-plane split`
- `## Batching, limits, queues, and backpressure`
- `## Measurement or validation plan`
- `## Open risks and non-goals`

#### Scenario: Estimating storage or queue cost

- GIVEN a maintainer needs to estimate log volume, queue capacity, or polling
  overhead
- WHEN they copy `templates/performance-sketch.md`
- THEN they can record rough numbers, units, and explicit limits without
  inventing a format first

### Requirement: Human Review Record For Performance Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt`
for the published guide and template.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/performance-review.md, templates/performance-sketch.md`
- `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
- `question 2: <pass|fail> - guide covers every required performance topic from Public Tiger Style Performance Review Guide under the designated headings`
- `question 3: <pass|fail> - template covers every required field, including resource budgets or limits per relevant resource, and every section stays a fill-in field or prompt rather than free-form prose`
- `question 4: <pass|fail> - overall guide guidance, not only per-section boilerplate, says static lints and benchmarks do not replace workload reasoning`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 4` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the docs are corrected.

#### Scenario: Reviewer signs off guide quality

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the required questions,
  the reviewed artifact paths, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN a deterministic check reads `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 4` are all `pass`
- AND `verdict` is `pass`
