# Design: Add Tiger Style Assertion Review Kit

## Context

The repo already enforces or documents pieces of Tiger Style assertion
behavior, but the public docs exposed from `README.md` still do not provide a
stable assertion review guide or a copyable template. Reviewers need a surface
that covers pair assertions, positive/negative space, compile-time checks,
fail-fast framing, and invalid-path tests without reducing the topic to raw
assertion-count metrics.

A lint-only approach is the wrong first step here. Much of Tiger Style's
assertion philosophy is about placement, boundary pairing, and review intent.
A doc-first review kit can improve reviews immediately without pretending the
whole philosophy is syntax-local.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for Tiger Style assertion review.
- Publish one copyable template for assertion planning and review notes.
- Make both artifacts discoverable from the README and contributor docs.
- Keep the content aligned with the existing safety spec instead of inventing a
  second assertion philosophy.

**Non-Goals:**
- New assertion lints in this change.
- Macro rewrites or assertion helper APIs.
- CI assertion gates or coverage thresholds.
- Replacing the downstream rollout guide with assertion docs.

## Decisions

### 1. Start with docs and templates, not lint expansion

**Choice:** Add a doc-first assertion review kit instead of new enforcement.

**Rationale:** Pair assertions, negative-space checks, and invalid-path tests
matter most during design and review. A public guide and template can make that
thinking reviewable now.

**Alternative:** Add more assertion lints first. Rejected because the highest-
value rules here are about placement and review reasoning, not only syntax.

### 2. Split explanatory guidance from copyable artifacts

**Choice:** Add `docs/assertion-review.md` for guidance and
`templates/assertion-checklist.md` for a reusable fill-in artifact.

**Rationale:** Readers need one surface that explains how to review assertions
and one surface they can copy into review notes or design docs.

**Alternative:** Put everything in `README.md` or the safety main spec.
Rejected because the README should stay a landing page and the main spec is a
contract, not a day-to-day review document.

### 3. Make the checklist map directly to Tiger Style assertion rules

**Choice:** The guide keeps these exact second-level headings, each exactly
once and in the listed order, with no extra second-level headings:
- `## Assertion targets: arguments, returns, invariants`
- `## Pair assertions across boundaries`
- `## Positive and negative space`
- `## Implication assertions and split checks`
- `## Compile-time assertions`
- `## Invalid-path testing`
- `## Reviewer prompts and failure handling`

The template keeps these exact second-level headings, each exactly once and in
the listed order, with no extra second-level headings:
- `## Function, boundary, or invariant under review`
- `## Assertions on inputs, outputs, and invariants`
- `## Paired assertion sites`
- `## Positive and negative space cases`
- `## Implications and split assertions`
- `## Compile-time assertions`
- `## Invalid-path tests`
- `## Open risks and follow-ups`

Within those headings, every template section is a fill-in field or prompt
rather than free-form narrative prose. Under every guide heading, the body text
includes at least one reviewer prompt or checklist question. The guide
explains why these checklist sections matter rather than presenting them as
slogans, with that rationale anchored in `## Reviewer prompts and failure
handling` plus overall guide guidance that assertion philosophy is not only
about density: assertions should fail fast on programmer errors, pair across
boundaries, and be backed by invalid-path testing.

Section-to-topic mapping stays explicit:
- `## Assertion targets: arguments, returns, invariants` owns function
  contract checks.
- `## Pair assertions across boundaries` owns caller/callee, write/read, and
  encode/decode pairing.
- `## Positive and negative space` owns valid/invalid boundary reasoning.
- `## Implication assertions and split checks` owns `if a { assert(b) }`
  patterns and splitting compound assertions.
- `## Compile-time assertions` owns constant relations and design-time checks.
- `## Invalid-path testing` owns explicit invalid-path and edge-case test
  planning.
- `## Reviewer prompts and failure handling` owns why the checklist matters,
  fail-fast framing, the rule that assertion philosophy is not only about raw
  density, and review cadence.

**Rationale:** The change should expose the missing public-surface guidance,
not paraphrase it loosely. Exact heading contracts make deterministic
verification possible and keep the guide/template aligned with the safety spec.

**Alternative:** Publish a shorter generic checklist. Rejected because it would
silently drop the specific Tiger Style rules this change is meant to expose.

### 4. Put discoverability in the README and contributor crosswalk

**Choice:** Update `README.md` to add these exact Markdown links as standalone lines at the top of the existing `## Documentation` section:
- `[Assertion review guide](docs/assertion-review.md)`
- `[Assertion checklist template](templates/assertion-checklist.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links as standalone lines near the top of the file after the existing intro paragraphs and before `## Source-of-truth map`:
- `[Assertion review guide](assertion-review.md)`
- `[Assertion checklist template](../templates/assertion-checklist.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

**Rationale:** The guide is only useful if readers can find it from the
entry points they already use, and contributor docs must show where the public
wording ultimately comes from.

**Alternative:** Link it only from the new guide or only from archived change
notes. Rejected because that keeps the assertion material effectively hidden.

### 5. Specify deterministic proof and the human/oracle gate explicitly

**Choice:** Deterministic verification uses exact line checks:
- `docs/assertion-review.md` must contain every exact heading from Decision 3
  exactly once, in order, as standalone `## ...` lines with no extra second-
  level headings.
- `templates/assertion-checklist.md` must contain every exact heading from
  Decision 3 exactly once, in order, as standalone `## ...` lines with no
  extra second-level headings.
- `README.md` must contain the exact Markdown-link lines
  `[Assertion review guide](docs/assertion-review.md)` and
  `[Assertion checklist template](templates/assertion-checklist.md)`, and
  deterministic proof must also show those target files exist.
- `docs/contributor-guide.md` must contain the exact Markdown-link lines
  `[Assertion review guide](assertion-review.md)`,
  `[Assertion checklist template](../templates/assertion-checklist.md)`, and
  `[Safety spec contract](../openspec/specs/safety/spec.md)`, and deterministic
  proof must also show `docs/assertion-review.md`,
  `templates/assertion-checklist.md`, and `openspec/specs/safety/spec.md`
  exist.
- deterministic proof must be saved at these paths so review can inspect the
  machine checks:
  - `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-guide-headings.txt`, containing the exact guide headings in order, one per line, after verification confirms they are the only second-level headings present
  - `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-template-headings.txt`, containing the exact template headings in order, one per line, after verification confirms they are the only second-level headings present
  - `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/readme-assertion-links.txt`, containing explicit `exists:...` lines for the linked files plus the two exact README link lines
  - `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/contributor-assertion-links.txt`, containing explicit `exists:...` lines for the linked files plus the three exact contributor-guide link lines
  - `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review-labels.txt`, containing these exact lines in order after exact-once validation succeeds: `reviewer role: Tiger Style maintainer:1`, `reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md:1`, `question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading:1`, `question 2: pass - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings:1`, `question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose:1`, `question 4: pass - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density:1`, `verdict: pass:1`
- `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
  must exist after the human/oracle checkpoint and must contain these exact
  labeled lines:
  - `reviewer role: Tiger Style maintainer`
  - `reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md`
  - `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
  - `question 2: <pass|fail> - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings`
  - `question 3: <pass|fail> - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose`
  - `question 4: <pass|fail> - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density`
  - `verdict: <pass|fail>`

The human/oracle checkpoint is owned by a Tiger Style maintainer. The saved
review record uses those exact lines and answer tokens, and each required label
appears exactly once. `human-assertion-review.txt` is created first during the
human/oracle checkpoint, and `human-assertion-review-labels.txt` is generated
only afterward from that saved file. `human-assertion-review-labels.txt`
records exact-once matches for the required `pass` lines listed above, so
deterministic proof shows both label presence and acceptance-critical answers.
For acceptance, questions 1 through 4 must all be `pass` and `verdict` must be
`pass`; any failed question blocks completion until the docs are corrected.

**Rationale:** Grep can prove the skeleton exists. It cannot prove the guide
avoids shallow or misleading assertion advice.

**Alternative:** Use deterministic checks only. Rejected because this change is
mostly about the quality and completeness of written guidance.

## Risks / Trade-offs

**[Docs without enforcement]** -> The guide may feel weaker than lints.
Mitigation: document explicitly that this is a public review kit, not the final
word on assertion automation.

**[Checklist drift]** -> The guide could drift from the main safety spec over
 time. Mitigation: keep `docs/contributor-guide.md` pointing back to
`openspec/specs/safety/spec.md` and verify the exact headings and link targets
 deterministically.
