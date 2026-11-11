# Add Tiger Style Bounds And Pacing Review Kit

## Why

Tiger Style wants systems to put limits on everything: size allocation up
front, keep resource growth bounded, buffer external events before processing,
and make the program run at its own pace instead of reacting directly to every
incoming event. This repo currently exposes only fragments of that philosophy
through specific lints and skill/spec prose. It does not yet ship a stable
public review guide or a copyable checklist for bounded allocation, queue
limits, backpressure, and self-paced external-event handling.

## What Changes

- This active change covers only the bounded allocation / external-event pacing
  sibling requested by the user; the assertion sibling is
  `openspec/changes/add-assertion-review-kit/` and the concurrency sibling is
  `openspec/changes/add-concurrency-state-review-kit/`.
- Add a public bounds-and-pacing review guide that turns Tiger Style bounded
  resource rules into a review checklist.
- Add a copyable bounds-and-pacing checklist template for allocation plans,
  queue bounds, batching, backpressure, and overload handling.
- Link the guide and template from the README, and link the guide, template,
  and the existing canonical main safety spec contract at
  `openspec/specs/safety/spec.md` from the contributor docs.
- Keep this change doc-first. It does not add allocator instrumentation,
  runtime pacing frameworks, or CI resource gates.

## Capabilities

### New Capabilities

- `bounds-and-pacing-review-kit`: Maintainers and consumers can review bounded
  allocation and external-event pacing with a stable Tiger Style checklist.
- `bounds-and-pacing-template`: Maintainers can capture allocation plans,
  queue/backpressure bounds, and pacing decisions in one reusable document.

## Impact

- **Files**: `README.md`, `docs/bounds-and-pacing-review.md`,
  `docs/contributor-guide.md`, `templates/bounds-and-pacing-checklist.md`,
  `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/specs/safety/spec.md`,
  `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/specs/documentation/spec.md`
- **APIs**: None.
- **Existing contracts**: `openspec/specs/safety/spec.md` already exists as
  the canonical main safety spec; contributor docs will link to that existing
  file rather than the change-local delta.
- **Dependencies**: None.
- **Testing**: Deterministic checks saved under `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/` for exact section headings with no extra second-level headings, explicit existence lines plus literal Markdown link destinations, and exact labeled-line acceptance at `human-bounds-review-labels.txt`, plus a saved human/oracle review record at `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt` with explicit pass/fail questions.
