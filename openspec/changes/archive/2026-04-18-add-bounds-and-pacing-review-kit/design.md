# Design: Add Tiger Style Bounds And Pacing Review Kit

## Context

The repo already documents bounded loops, channels, and collection growth in
pieces, but it still lacks a stable public guide for bounded allocation and
external-event pacing. Reviewers need one place that turns startup sizing,
queue limits, batching, backpressure, and self-paced event handling into a
repeatable checklist and a copyable planning template.

A lint-only approach is not enough. Allocation lifetime, event ownership, and
pacing boundaries depend on architecture and workload intent. A doc-first
review kit can make that reasoning reviewable now.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for bounded allocation and pacing review.
- Publish one copyable template for allocation/pacing planning notes.
- Make both artifacts discoverable from the README and contributor docs.
- Keep the content aligned with the existing safety spec.

**Non-Goals:**
- New runtime frameworks or allocators.
- New CI resource gates.
- Replacing lint rollout docs with pacing docs.
- Consumer-specific numeric thresholds.

## Decisions

### 1. Start with docs and templates, not runtime machinery

**Choice:** Add a doc-first bounds-and-pacing review kit instead of new
runtime enforcement.

**Rationale:** Tiger Style boundedness rules are strongest at design time:
allocation lifetime, queue sizing, batching, and self-paced event ownership are
review questions first.

**Alternative:** Add more enforcement first. Rejected because the highest-value
rules here are architectural and workload-dependent.

### 2. Split explanatory guidance from copyable artifacts

**Choice:** Add `docs/bounds-and-pacing-review.md` for guidance and
`templates/bounds-and-pacing-checklist.md` for a reusable fill-in artifact.

**Rationale:** Readers need one surface that explains how to review bounds and
pacing and one surface they can copy into design or review notes.

**Alternative:** Put everything in `README.md` or the main safety spec.
Rejected because the README should stay a landing page and the main spec is a
contract, not the review document.

### 3. Make the checklist map directly to Tiger Style boundedness rules

**Choice:** The guide keeps these exact second-level headings, each exactly
once and in the listed order, with no extra second-level headings:
- `## Allocation lifetime and ownership`
- `## Startup sizing and post-init bounds`
- `## Queues, buffers, and backpressure`
- `## External-event pacing`
- `## Batch and work budgeting`
- `## Overload and failure behavior`
- `## Reviewer prompts and escape hatches`

The template keeps these exact second-level headings, each exactly once and in
the listed order, with no extra second-level headings:
- `## Component or boundary`
- `## Allocation plan`
- `## Startup sizing and post-init limits`
- `## Queue and buffer bounds`
- `## External-event intake and pacing`
- `## Batch and work budget`
- `## Overload and failure behavior`
- `## Open risks and follow-ups`

Within those headings, every template section is a fill-in field or prompt
rather than free-form narrative prose. Under every guide heading, the body text
includes at least one reviewer prompt or checklist question and explains why
the checklist matters. The `## External-event pacing` section says systems
should buffer external events and run at their own pace instead of reacting
directly to every event.

Template section-to-topic mapping stays explicit:
- `## Component or boundary` owns the component or boundary under review.
- `## Allocation plan` owns allocation lifetime, ownership, and who may grow
  or retain resources.
- `## Startup sizing and post-init limits` owns startup sizing and bounded
  post-init behavior.
- `## Queue and buffer bounds` owns explicit queue and buffer bounds plus
  backpressure choices.
- `## External-event intake and pacing` owns buffering external events before
  processing and running at the system's own pace.
- `## Batch and work budget` owns per-batch and per-tick work budgeting.
- `## Overload and failure behavior` owns overload handling and failure mode
  choices.
- `## Open risks and follow-ups` owns residual risks and deferred follow-up
  work.

Section-to-topic mapping stays explicit:
- `## Allocation lifetime and ownership` owns allocation lifetime, ownership,
  and who may grow or retain resources.
- `## Startup sizing and post-init bounds` owns startup sizing and bounded
  post-init behavior.
- `## Queues, buffers, and backpressure` owns explicit bounds, queue growth,
  and backpressure decisions.
- `## External-event pacing` owns the rule that the system should buffer
  external events before processing and run at its own pace.
- `## Batch and work budgeting` owns per-batch and per-tick work budgeting.
- `## Overload and failure behavior` owns overload handling and fail-fast or
  degraded-mode decisions.
- `## Reviewer prompts and escape hatches` owns audited exceptions and review
  cadence.

**Rationale:** Exact heading contracts make deterministic verification possible
and keep the guide/template aligned with the safety spec.

**Alternative:** Publish a shorter generic checklist. Rejected because it would
silently drop the specific Tiger Style rules this change is meant to expose.

### 4. Put discoverability in the README and contributor crosswalk

**Choice:** Update `README.md` to add these exact Markdown links:
- `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)`
- `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links:
- `[Bounds and pacing review guide](bounds-and-pacing-review.md)`
- `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

**Rationale:** The guide is only useful if readers can find it from the entry
points they already use, and contributor docs must show where the public
wording ultimately comes from.

**Alternative:** Link it only from the new guide. Rejected because that keeps
bounds/pacing material effectively hidden.

### 5. Specify deterministic proof and the human/oracle gate explicitly

**Choice:** Deterministic verification uses exact line checks:
- `docs/bounds-and-pacing-review.md` must contain every exact heading from
  Decision 3 exactly once, in order, as standalone `## ...` lines with no
  extra second-level headings.
- `templates/bounds-and-pacing-checklist.md` must contain every exact heading
  from Decision 3 exactly once, in order, as standalone `## ...` lines with no
  extra second-level headings.
- `README.md` must contain the exact Markdown-link lines
  `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)` and
  `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`, and
  deterministic proof must also show those target files exist.
- `docs/contributor-guide.md` must contain the exact Markdown-link lines
  `[Bounds and pacing review guide](bounds-and-pacing-review.md)`,
  `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`,
  and `[Safety spec contract](../openspec/specs/safety/spec.md)`, and
  deterministic proof must also show `docs/bounds-and-pacing-review.md`,
  `templates/bounds-and-pacing-checklist.md`, and `openspec/specs/safety/spec.md`
  exist.
- deterministic proof must be saved at these paths so review can inspect the
  machine checks:
  - `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-guide-headings.txt`, containing the exact guide headings in order, one per line, after verification confirms they are the only second-level headings present
  - `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-template-headings.txt`, containing the exact template headings in order, one per line, after verification confirms they are the only second-level headings present
  - `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/readme-bounds-links.txt`, containing these exact lines in order: `exists:docs/bounds-and-pacing-review.md`, `exists:templates/bounds-and-pacing-checklist.md`, `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)`, `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)`
  - `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/contributor-bounds-links.txt`, containing these exact lines in order: `exists:docs/bounds-and-pacing-review.md`, `exists:templates/bounds-and-pacing-checklist.md`, `exists:openspec/specs/safety/spec.md`, `[Bounds and pacing review guide](bounds-and-pacing-review.md)`, `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`, `[Safety spec contract](../openspec/specs/safety/spec.md)`
  - `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review-labels.txt`, containing these exact lines in order after exact-once validation succeeds: `reviewer role: Tiger Style maintainer:1`, `reviewed artifact paths: docs/bounds-and-pacing-review.md, templates/bounds-and-pacing-checklist.md:1`, `question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading:1`, `question 2: pass - guide covers every required bounds-and-pacing topic from Public Tiger Style Bounds And Pacing Review Guide under the designated headings:1`, `question 3: pass - template covers every required field, including backpressure choices, and every section stays a fill-in field or prompt rather than free-form prose:1`, `question 4: pass - the External-event pacing guidance says systems buffer external events and run at their own pace instead of reacting directly to every event:1`, `verdict: pass:1`
- `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
  must exist after the human/oracle checkpoint and must contain these exact
  labeled lines:
  - `reviewer role: Tiger Style maintainer`
  - `reviewed artifact paths: docs/bounds-and-pacing-review.md, templates/bounds-and-pacing-checklist.md`
  - `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
  - `question 2: <pass|fail> - guide covers every required bounds-and-pacing topic from Public Tiger Style Bounds And Pacing Review Guide under the designated headings`
  - `question 3: <pass|fail> - template covers every required field, including backpressure choices, and every section stays a fill-in field or prompt rather than free-form prose`
  - `question 4: <pass|fail> - the External-event pacing guidance says systems buffer external events and run at their own pace instead of reacting directly to every event`
  - `verdict: <pass|fail>`

The human/oracle checkpoint is owned by a Tiger Style maintainer. The saved
review record uses those exact lines and answer tokens, and each required
label appears exactly once.
`openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review-labels.txt`
records exact-once matches for the required `pass` lines listed above, so
saved deterministic proof shows both label presence and acceptance-critical
answers. For acceptance, questions 1 through 4 must all be `pass` and
`verdict` must be `pass`; any failed question blocks completion until the docs
are corrected.

**Rationale:** Grep can prove the skeleton exists. It cannot prove the guide
keeps the pacing philosophy intact.

**Alternative:** Use deterministic checks only. Rejected because this change is
mostly about the quality and completeness of written guidance.

## Risks / Trade-offs

**[Docs without enforcement]** -> The guide may feel weaker than runtime
instrumentation. Mitigation: document explicitly that this is a review kit, not
a runtime framework.

**[Checklist drift]** -> The guide could drift from the main safety spec over
 time. Mitigation: keep `docs/contributor-guide.md` pointing back to
`openspec/specs/safety/spec.md` and verify headings and links deterministically.
