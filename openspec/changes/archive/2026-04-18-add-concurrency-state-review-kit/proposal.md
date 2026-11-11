# Add Tiger Style Concurrency And Distributed-State Review Kit

## Why

Tiger Style wants concurrency rules to be explicit: shared-state ownership,
lock ordering, deterministic peer ordering, typed or state-machine transitions,
narrow quarantined unsafety, and loud escape hatches. This repo currently
captures only part of that story through one shipped lint and skill/spec prose.
It does not yet ship a stable public review guide or a copyable checklist for
concurrency and distributed-state rules.

## What Changes

- This active change is the stronger concurrency / distributed-state sibling
  requested by the user; the assertion sibling is
  `openspec/changes/add-assertion-review-kit/` and the bounded-allocation
  sibling is `openspec/changes/add-bounds-and-pacing-review-kit/`.
- Add a public concurrency and distributed-state review guide that turns Tiger
  Style concurrency rules into a review checklist, including explicit reviewer
  prompts and failure-handling guidance.
- Use that review kit to encode the stronger concurrency and distributed-state
  rules in a public, reviewable form: explicit shared-state ownership,
  deterministic ordering, typed transitions, retry/idempotence assumptions,
  narrow unsafe boundaries, and failure-handling guidance.
- Add a copyable concurrency and distributed-state checklist template for
  shared-state inventory, ordering rules, state transitions, retry/replay
  assumptions, unsafe boundaries, and open risks/follow-ups.
- Link the guide and template from the README, and link the guide, template,
  and the existing canonical main concurrency spec contract at
  `openspec/specs/concurrency/spec.md` from the contributor docs.
- Keep this change doc-first. It does not add new lock-analysis lints,
  state-machine macros, or distributed test harnesses.

## Capabilities

### New Capabilities

- `concurrency-state-review-kit`: Maintainers and consumers can review
  concurrency and distributed-state code with a stable Tiger Style checklist.
- `concurrency-state-template`: Maintainers can capture ordering, transitions,
  retry/replay assumptions, and unsafe boundaries in one reusable document.

## Impact

- **Files**: `README.md`, `docs/concurrency-state-review.md`,
  `docs/contributor-guide.md`, `templates/concurrency-state-checklist.md`,
  `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/specs/concurrency/spec.md`,
  `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/specs/documentation/spec.md`
- **APIs**: None.
- **Existing contracts**: `openspec/specs/concurrency/spec.md` already exists
  as the canonical main concurrency spec; contributor docs will link to that
  existing file rather than the change-local delta.
- **Dependencies**: None.
- **Testing**: Deterministic checks saved under `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/` for exact section headings with no extra second-level headings, literal Markdown link destinations, explicit existence lines, and exact labeled-line acceptance at `human-concurrency-review-labels.txt`, plus a saved human/oracle review record at `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt` with explicit pass/fail questions, including canonical-alignment review against `openspec/specs/concurrency/spec.md`.
