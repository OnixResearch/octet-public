# Concurrency Specification

## Purpose

Tiger Style concurrency requirements for lock ordering, peer-lock
acquisition, and narrow audited unsafety boundaries.
## Requirements
### Requirement: Unsafe Quarantine Guidance In Skill

The Tiger Style skill MUST include the reference guidance that
safety-critical concurrency internals may use `unsafe`, but public APIs
must remain safe, small, and auditable.

The skill MUST distinguish between:

- acceptable localized internal unsafety behind a safe wrapper, and
- unsafe escape hatches that must be explicit, grep-able, and dangerous by name

#### Scenario: Reviewing a lock primitive

- GIVEN a concurrency primitive with internal raw synchronization
- WHEN an agent follows the skill
- THEN it expects the public API to stay safe and the unsafety to be localized to a narrow implementation boundary

### Requirement: Multiple Locks Without Documented Ordering

The lint MUST be able to warn on struct definitions containing 2 or more fields
of type `Mutex<_>`, `RwLock<_>`, `tokio::sync::Mutex<_>`, or
`tokio::sync::RwLock<_>` when there is no comment in the struct body
documenting lock acquisition order.

The lint MUST accept a comment containing "lock order" or "ordering"
anywhere in the struct definition.

This lint exists because clankers has structs like `LivenessTracker`
with `last_output: Mutex<Instant>` and `state: Mutex<HealthState>`,
and `CredentialManager` with `credential: Mutex<Credential>` and
`refresh_guard: Mutex<()>`. Undocumented lock ordering leads to
deadlocks that only appear under production concurrency.

#### Scenario: Two mutexes without ordering comment

- GIVEN:
  ```rust
  struct Tracker {
      count: Mutex<u64>,
      state: Mutex<State>,
  }
  ```
- WHEN the lint runs
- THEN it warns: "struct has 2 Mutex fields without documented lock ordering"

#### Scenario: Two mutexes with ordering comment

- GIVEN:
  ```rust
  struct Tracker {
      // Lock ordering: always acquire `count` before `state`
      count: Mutex<u64>,
      state: Mutex<State>,
  }
  ```
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Single mutex

- GIVEN a struct with exactly one `Mutex` field
- WHEN the lint runs
- THEN it does NOT fire (no ordering concern)

### Requirement: Public Tiger Style Concurrency And Distributed-State Review Guide

The project MUST ship a public concurrency and distributed-state review guide
at `docs/concurrency-state-review.md` that translates Tiger Style concurrency
rules into a repeatable review checklist.

That guide MUST cover:

- why these checklist sections matter instead of presenting them as slogans
- shared-state inventory and ownership boundaries
- lock ordering and deterministic peer ordering
- typed or state-machine transitions and invariants
- message ordering, retries, replay assumptions, and idempotence
- narrow unsafe quarantine behind safe wrappers
- loud, grep-able escape hatches for dangerous exceptions
- failure handling and escalation guidance for concurrency review findings

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question and a short
explanation of why that section matters.

That guide MUST keep these exact second-level headings:

- `## Shared-state inventory and ownership`
- `## Lock ordering and peer ordering`
- `## State-machine transitions and invariants`
- `## Message ordering, retries, and idempotence`
- `## Unsafe quarantine and escape hatches`
- `## Reviewer prompts and failure handling`

Section ownership stays explicit:

- `## Shared-state inventory and ownership` owns shared mutable state and
  ownership boundaries.
- `## Lock ordering and peer ordering` owns documented lock order and
  deterministic peer/resource ordering.
- `## State-machine transitions and invariants` owns typed transitions,
  invariant checks, and invalid-state handling.
- `## Message ordering, retries, and idempotence` owns delivery, replay,
  retry, and idempotence assumptions.
- `## Unsafe quarantine and escape hatches` owns narrow unsafe boundaries and
  loud, grep-able escape hatches.
- `## Reviewer prompts and failure handling` owns reviewer disposition,
  failure-handling guidance, and escalation paths when concurrency review
  finds unresolved risk.

#### Scenario: Guide keeps required headings and order

- GIVEN the guide has been drafted
- WHEN deterministic verification reads `docs/concurrency-state-review.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Reviewing lock order or protocol transitions

- GIVEN a maintainer is reviewing concurrent shared state or a protocol
  transition boundary
- WHEN they follow `docs/concurrency-state-review.md`
- THEN the guide prompts explicit ordering, transition, and unsafe-boundary
  checks
- AND it keeps deterministic peer ordering and replay assumptions visible

### Requirement: Copyable Concurrency-State Template

The project MUST ship a copyable concurrency-state checklist template at
`templates/concurrency-state-checklist.md`.

That template MUST include fill-in fields for:

- the component or protocol boundary under review
- shared-state inventory
- lock and peer ordering
- state-machine transitions and invariants
- delivery, retry, and idempotence assumptions
- unsafe boundary and escape hatches
- open risks and follow-ups

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template.

That template MUST keep these exact second-level headings:

- `## Component or protocol boundary`
- `## Shared-state inventory`
- `## Lock and peer ordering`
- `## State-machine transitions and invariants`
- `## Delivery, retry, and idempotence assumptions`
- `## Unsafe boundary and escape hatches`
- `## Open risks and follow-ups`

#### Scenario: Template keeps required headings and order

- GIVEN the template has been drafted
- WHEN deterministic verification reads
  `templates/concurrency-state-checklist.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Planning ordering and retry assumptions

- GIVEN a maintainer needs to review concurrency or distributed-state behavior
- WHEN they copy `templates/concurrency-state-checklist.md`
- THEN they can record ordering rules, retry/replay assumptions, and unsafe
  boundaries without inventing a format first

### Requirement: Saved Deterministic Proof Artifacts For Concurrency Kit

The implementation MUST save deterministic verification artifacts at these
paths:

- `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-guide-headings.txt`
- `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-template-headings.txt`
- `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/readme-concurrency-links.txt`
- `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/contributor-concurrency-links.txt`
- `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review-labels.txt`

Those artifacts MUST capture these exact saved outputs:

- `concurrency-guide-headings.txt` prints the full list of detected guide
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `concurrency-template-headings.txt` prints the full list of detected
  template second-level headings, one per line, after verification confirms
  that list exactly matches the required headings in order with no extras.
- `readme-concurrency-links.txt` prints
  `exists:docs/concurrency-state-review.md`,
  `exists:templates/concurrency-state-checklist.md`, then the two exact README
  link lines.
- `contributor-concurrency-links.txt` prints
  `exists:docs/concurrency-state-review.md`,
  `exists:templates/concurrency-state-checklist.md`,
  `exists:openspec/specs/concurrency/spec.md`, then the three exact
  contributor-doc link lines.
- `human-concurrency-review-labels.txt` prints these exact lines in order after
  exact-once validation succeeds:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/concurrency-state-review.md, templates/concurrency-state-checklist.md:1`
  - `question 1: pass - each required guide heading explains why that section matters and includes reviewer prompts:1`
  - `question 2: pass - guide covers every required concurrency topic from Public Tiger Style Concurrency And Distributed-State Review Guide under the designated headings and stays aligned with openspec/specs/concurrency/spec.md:1`
  - `question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose:1`
  - `question 4: pass - overall guide guidance says unsafe stays behind narrow safe wrappers, dangerous escape hatches stay grep-able and explicit, and this document is a review kit rather than the full enforcement story:1`
  - `verdict: pass:1`

#### Scenario: Saved guide heading artifact keeps exact heading inventory

- GIVEN deterministic verification checks `docs/concurrency-state-review.md`
- WHEN it saves `concurrency-guide-headings.txt`
- THEN the artifact records the full verified heading inventory with no extras

#### Scenario: Saved template heading artifact keeps exact heading inventory

- GIVEN deterministic verification checks
  `templates/concurrency-state-checklist.md`
- WHEN it saves `concurrency-template-headings.txt`
- THEN the artifact records the full verified heading inventory with no extras

#### Scenario: Saved README link artifact keeps existence and exact link lines

- GIVEN deterministic verification checks `README.md`
- WHEN it saves `readme-concurrency-links.txt`
- THEN the artifact records the required file existence lines and exact README
  link lines

#### Scenario: Saved contributor link artifact keeps existence and exact link lines

- GIVEN deterministic verification checks `docs/contributor-guide.md`
- WHEN it saves `contributor-concurrency-links.txt`
- THEN the artifact records the required existence lines and exact
  contributor-doc link lines

#### Scenario: Saved review-label artifact keeps exact label counts

- GIVEN deterministic verification checks
  `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
- WHEN it saves `human-concurrency-review-labels.txt`
- THEN the artifact records the exact required label-count lines in order

### Requirement: Human Review Record For Concurrency Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
for the published guide and template. Each required label below MUST appear
exactly once in that file.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/concurrency-state-review.md, templates/concurrency-state-checklist.md`
- `question 1: <pass|fail> - each required guide heading explains why that section matters and includes reviewer prompts`
- `question 2: <pass|fail> - guide covers every required concurrency topic from Public Tiger Style Concurrency And Distributed-State Review Guide under the designated headings and stays aligned with openspec/specs/concurrency/spec.md`
- `question 3: <pass|fail> - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose`
- `question 4: <pass|fail> - overall guide guidance says unsafe stays behind narrow safe wrappers, dangerous escape hatches stay grep-able and explicit, and this document is a review kit rather than the full enforcement story`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 4` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the docs are corrected.

#### Scenario: Reviewer signs off concurrency coverage

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  paths, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 4` are all `pass`
- AND `verdict` is `pass`

