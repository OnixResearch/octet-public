# Design: Add Tiger Style Concurrency And Distributed-State Review Kit

## Context

The repo already documents lock ordering and narrow unsafe boundaries in pieces,
but it still lacks a stable public guide for stronger concurrency and
distributed-state review. Reviewers need one place that turns shared-state
inventory, ordering rules, typed transitions, retry/replay assumptions, and
unsafe quarantine into a repeatable checklist and a copyable planning
template.

A lint-only approach is not enough. Concurrency and distributed-state rules are
architectural and protocol-level. A doc-first review kit can make that
reasoning reviewable now.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for concurrency and distributed-state review.
- Publish one copyable template for concurrency-state planning notes.
- Make both artifacts discoverable from the README and contributor docs.
- Keep the content aligned with the existing concurrency spec.

**Non-Goals:**
- New lock-analysis lints in this change.
- State-machine DSLs or macros.
- Distributed test harnesses or model-checker integration.
- Replacing rollout docs with concurrency docs.

## Decisions

### 1. Start with docs and templates, not new enforcement

**Choice:** Add a doc-first concurrency-state review kit instead of new
analysis machinery.

**Rationale:** Shared-state ownership, peer ordering, transition modeling, and
unsafe quarantine matter first as design and review questions.

**Alternative:** Add more enforcement first. Rejected because the highest-value
rules here are protocol- and architecture-level.

### 2. Split explanatory guidance from copyable artifacts

**Choice:** Add `docs/concurrency-state-review.md` for guidance and
`templates/concurrency-state-checklist.md` for a reusable fill-in artifact.

**Rationale:** Readers need one surface that explains how to review
concurrency/distributed-state code and one surface they can copy into design or
review notes.

**Alternative:** Put everything in `README.md` or the main concurrency spec.
Rejected because the README should stay a landing page and the main spec is a
contract, not the review document.

### 3. Make the checklist map directly to Tiger Style concurrency rules

**Choice:** The guide keeps these exact second-level headings, each exactly
once and in the listed order, with no extra second-level headings:
- `## Shared-state inventory and ownership`
- `## Lock ordering and peer ordering`
- `## State-machine transitions and invariants`
- `## Message ordering, retries, and idempotence`
- `## Unsafe quarantine and escape hatches`
- `## Reviewer prompts and failure handling`

The template keeps these exact second-level headings, each exactly once and in
the listed order, with no extra second-level headings:
- `## Component or protocol boundary`
- `## Shared-state inventory`
- `## Lock and peer ordering`
- `## State-machine transitions and invariants`
- `## Delivery, retry, and idempotence assumptions`
- `## Unsafe boundary and escape hatches`
- `## Open risks and follow-ups`

Within those headings, every template section is a fill-in field or prompt
rather than free-form narrative prose. Under every guide heading, the body text
includes at least one reviewer prompt or checklist question and explains why
that section matters. The guide also says, in its overall guidance rather than
per-heading boilerplate, that unsafe must stay behind narrow safe wrappers,
that dangerous escape hatches should stay grep-able and explicit, and that this
document is a review kit rather than the full enforcement story.

Section-to-topic mapping stays explicit:
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
  failure-handling guidance, and escalation paths.

**Rationale:** Exact heading contracts make deterministic verification possible
and keep the guide/template aligned with the concurrency spec.

**Alternative:** Publish a shorter generic checklist. Rejected because it would
silently drop the specific Tiger Style rules this change is meant to expose.

### 4. Put discoverability in the README and contributor crosswalk

**Choice:** Update `README.md` to add these exact Markdown links:
- `[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)`
- `[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links:
- `[Concurrency and distributed-state review guide](concurrency-state-review.md)`
- `[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)`
- `[Concurrency spec contract](../openspec/specs/concurrency/spec.md)`

**Rationale:** The guide is only useful if readers can find it from the entry
points they already use, and contributor docs must show where the public
wording ultimately comes from.

**Alternative:** Link it only from the new guide. Rejected because that keeps
the concurrency material effectively hidden.

### 5. Specify deterministic proof and the human/oracle gate explicitly

**Choice:** Deterministic verification uses exact line checks:
- `docs/concurrency-state-review.md` must contain every exact heading from
  Decision 3 exactly once, in order, as standalone `## ...` lines with no
  extra second-level headings.
- `templates/concurrency-state-checklist.md` must contain every exact heading
  from Decision 3 exactly once, in order, as standalone `## ...` lines with no
  extra second-level headings.
- `README.md` must contain the exact Markdown-link lines
  `[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)`
  and `[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)`,
  and deterministic proof must also show those target files exist.
- `docs/contributor-guide.md` must contain the exact Markdown-link lines
  `[Concurrency and distributed-state review guide](concurrency-state-review.md)`,
  `[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)`,
  and `[Concurrency spec contract](../openspec/specs/concurrency/spec.md)`, and
  deterministic proof must also show `docs/concurrency-state-review.md`,
  `templates/concurrency-state-checklist.md`, and
  `openspec/specs/concurrency/spec.md` exist.
- deterministic proof must be saved at these paths so review can inspect the
  machine checks:
  - `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-guide-headings.txt`, containing the exact guide headings in order, one per line, after verification confirms they are the only second-level headings present
  - `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-template-headings.txt`, containing the exact template headings in order, one per line, after verification confirms they are the only second-level headings present
  - `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/readme-concurrency-links.txt`, containing explicit `exists:...` lines for the linked files plus the two exact README link lines
  - `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/contributor-concurrency-links.txt`, containing explicit `exists:...` lines for the linked files plus the three exact contributor-guide link lines
  - `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review-labels.txt`, containing these exact lines in order after exact-once validation succeeds: `reviewer role: Tiger Style maintainer:1`, `reviewed artifact paths: docs/concurrency-state-review.md, templates/concurrency-state-checklist.md:1`, `question 1: pass - each required guide heading explains why that section matters and includes reviewer prompts:1`, `question 2: pass - guide covers every required concurrency topic from Public Tiger Style Concurrency And Distributed-State Review Guide under the designated headings and stays aligned with openspec/specs/concurrency/spec.md:1`, `question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose:1`, `question 4: pass - overall guide guidance says unsafe stays behind narrow safe wrappers, dangerous escape hatches stay grep-able and explicit, and this document is a review kit rather than the full enforcement story:1`, `verdict: pass:1`
- `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
  must exist after the human/oracle checkpoint and must contain these exact
  labeled lines:
  - `reviewer role: Tiger Style maintainer`
  - `reviewed artifact paths: docs/concurrency-state-review.md, templates/concurrency-state-checklist.md`
  - `question 1: <pass|fail> - each required guide heading explains why that section matters and includes reviewer prompts`
  - `question 2: <pass|fail> - guide covers every required concurrency topic from Public Tiger Style Concurrency And Distributed-State Review Guide under the designated headings and stays aligned with openspec/specs/concurrency/spec.md`
  - `question 3: <pass|fail> - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose`
  - `question 4: <pass|fail> - overall guide guidance says unsafe stays behind narrow safe wrappers, dangerous escape hatches stay grep-able and explicit, and this document is a review kit rather than the full enforcement story`
  - `verdict: <pass|fail>`

The human/oracle checkpoint is owned by a Tiger Style maintainer. The saved
review record uses those exact lines and answer tokens. Review question 2 is
the required acceptance check tying the new guide/template back to the existing
canonical concurrency contract at `openspec/specs/concurrency/spec.md`.
`human-concurrency-review-labels.txt` records exact-once matches for the
required `pass` lines listed above, so saved deterministic proof shows both
label presence and acceptance-critical answers. For acceptance, questions 1
through 4 must all be `pass` and `verdict` must be `pass`; any failed question
blocks completion until the docs are corrected.

**Rationale:** Grep can prove the skeleton exists. It cannot prove the guide
keeps the concurrency philosophy intact.

**Alternative:** Use deterministic checks only. Rejected because this change is
mostly about the quality and completeness of written guidance.

## Risks / Trade-offs

**[Docs without enforcement]** -> The guide may feel weaker than new lints or
harnesses. Mitigation: document explicitly that this is a review kit, not the
full enforcement story.

**[Checklist drift]** -> The guide could drift from the main concurrency spec
over time. Mitigation: keep `docs/contributor-guide.md` pointing back to
`openspec/specs/concurrency/spec.md` and verify headings and links
deterministically.
