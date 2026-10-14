# Performance Guidance Specification

## Purpose

Performance guidance for Tiger Style materials, including agent-facing skill
coverage of design-time performance rules.
## Requirements
### Requirement: Full Performance Principle Coverage In Skill

The Tiger Style skill MUST cover the full performance guidance from the
reference, not only the high-level themes.

The skill MUST include:

- design for performance early
- napkin math
- analysis of bandwidth AND latency
- separation of control plane and data plane
- batching
- resource priority ordering
- optimization by cost × frequency
- large CPU work chunks
- hot-loop extraction with primitive arguments
- predictability for cache and branch behavior
- reduced dependence on compiler optimization luck

#### Scenario: Reviewing a hot path

- GIVEN an agent is reviewing a performance-sensitive path
- WHEN it follows the skill
- THEN it is prompted to reason about latency, bandwidth, frequency, batching, and predictability rather than only micro-optimizations

### Requirement: Napkin Math Example Or Equivalent Pointer

The skill MUST include either a worked napkin-math example or an explicit,
nearby pointer to the worked example in the reference.

The skill MUST explain that napkin math is used for early bottleneck
identification, sanity checks, and rough cost estimation.

#### Scenario: Estimating storage cost

- GIVEN an agent needs to evaluate a logging design
- WHEN it reads the skill
- THEN it can discover that rough throughput and storage calculations are part of Tiger Style, not optional extra analysis

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

### Requirement: Clone Then Borrow Lint [r[clone-then-borrow-lint]]

The lint MUST be able to warn when code clones a value and immediately takes a shared borrow of the clone result instead of borrowing the original value or using an explicit borrowed adapter.

The first implementation MUST cover direct expressions shaped like `&value.clone()` and `&(value.clone())` when the clone call resolves to `Clone::clone` and the clone result is not otherwise consumed as owned state.

The lint MUST emit advisory or warning-level diagnostics first, with review-required suggestions such as borrowing the original value or using `.as_ref()`, `.as_str()`, `.as_slice()`, or `.as_path()` where appropriate.

The lint MUST NOT warn on clones that are moved into ownership-consuming APIs, stored for later independent lifetime, independently mutated, returned, sent across thread/task boundaries, or generated by macro expansion unless future fixtures explicitly prove safe behavior.

#### Scenario: Immediate String clone then borrow [r[clone-then-borrow-lint.scenario.string-clone-borrow]]

- GIVEN `fn log_name(name: &String) { emit(&name.clone()); }`
- WHEN the lint runs
- THEN it warns that cloning and immediately borrowing the `String` hides the borrowed contract and can allocate unnecessarily
- AND the diagnostic suggests borrowing the original value or using a borrowed adapter with review-required applicability

#### Scenario: Immediate Vec clone then borrow [r[clone-then-borrow-lint.scenario.vec-clone-borrow]]

- GIVEN `fn checksum(bytes: &Vec<u8>) -> u32 { sum(&bytes.clone()) }`
- WHEN the lint runs
- THEN it warns that cloning and immediately borrowing the `Vec` pays allocation/copy cost without transferring ownership

#### Scenario: Owned clone transfer is accepted [r[clone-then-borrow-lint.scenario.owned-clone-transfer-accepted]]

- GIVEN `let owned = name.clone(); queue.push(owned);`
- WHEN the lint runs
- THEN it does NOT warn because the clone is transferred as owned state

#### Scenario: Stored or lifetime-detaching clone is accepted [r[clone-then-borrow-lint.scenario.stored-detach-accepted]]

- GIVEN a clone is stored in a struct, returned, moved into a task, or used to detach from a lock guard or short-lived borrow
- WHEN the lint runs
- THEN it does NOT warn in the first implementation scope because the clone may be the intended ownership boundary

#### Scenario: Expansion boundary is skipped [r[clone-then-borrow-lint.scenario.expansion-skip]]

- GIVEN a macro expansion generates a borrowed clone expression
- WHEN the lint runs
- THEN it does NOT warn unless a future fixture proves that the source span points at user-owned code with an actionable replacement

### Requirement: Content Identity Lint Cache [r[content-identity-lint-cache]]

The system MUST support an optional deterministic lint-result cache keyed by Octet version, rustc/toolchain identity, lint config hash, crate graph hash, lint ID, and semantic item identity. The cache MUST be disabled or bypassed for findings that cannot prove deterministic inputs.

#### Scenario: Unchanged item uses cached finding [r[content-identity-lint-cache.scenario.unchanged-item-uses-cached-finding]]

- GIVEN an item identity, config hash, crate graph hash, and toolchain identity are unchanged between runs
- WHEN `cargo octet check --cache` runs again
- THEN deterministic findings for that item are loaded from cache and reported with cache-hit metadata

#### Scenario: Config change invalidates cache [r[content-identity-lint-cache.scenario.config-change-invalidates-cache]]

- GIVEN the lint config hash changes between two runs
- WHEN `cargo octet check --cache` runs
- THEN previous cached findings are not reused for affected lint IDs

#### Scenario: Non-deterministic input bypasses cache [r[content-identity-lint-cache.scenario.non-deterministic-input-bypasses-cache]]

- GIVEN an item lacks semantic identity or depends on unsupported macro expansion context
- WHEN the cache key is computed
- THEN the item is marked cache-ineligible and linting runs normally

### Requirement: Pure Result Cache [r[pure-result-cache]]

The system MUST support deterministic pure-result caching for eligible lint, check, documentation, and receipt-generation outputs. Cache keys MUST include command identity, normalized input hashes, configuration hash, tool revision, and an explicit environment allowlist. Commands with undeclared network, secret, clock, randomness, or broad filesystem effects MUST be uncached or partitioned by declared effect identity.

#### Scenario: Pure cache hit reuses matching result [r[pure-result-cache.scenario.cache-hit-reuses-matching-result]]

- GIVEN a deterministic command has the same command identity, input hashes, config hash, tool revision, and environment allowlist as a previous run
- WHEN the pure-result cache is queried
- THEN the cache may reuse the previous result
- AND the hit receipt names the cache key components and stored output hash

#### Scenario: Environment drift invalidates cache [r[pure-result-cache.scenario.environment-drift-invalidates-cache]]

- GIVEN a deterministic command's input hashes are unchanged but the declared environment allowlist or tool revision changes
- WHEN the pure-result cache is queried
- THEN the previous result is not reused as a valid hit
- AND the miss receipt identifies the changed key component without dumping the full environment
