# Design: Add Tiger Style Resilience Review Kit

## Context

Tiger Style already teaches maintainers to avoid hidden panics, silent failure
paths, and unbounded growth. Those rules matter, but they do not yet add up to
one public resilience review surface. The missing public piece is a guide that
turns repeated incidents and scary retries into design review questions, plus a
copyable template that records the five-whys chain and the surrounding
resilience assumptions.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for resilience review.
- Publish one copyable five-whys architecture template.
- Make both artifacts discoverable from the README and contributor docs.
- Keep the content focused on reviewable resilience decisions rather than new
  runtime machinery.

**Non-Goals:**
- New retry libraries, queue frameworks, or incident-management tools.
- Chaos-engineering automation.
- Runtime enforcement of fail-open or fail-closed policies.
- Replacing the existing safety or concurrency specs with incident prose.

## Decisions

### 1. Use a resilience guide and a five-whys template, not new runtime code

**Choice:** Add `docs/resilience-review.md` and
`templates/five-whys-architecture.md` as doc-first review surfaces.

**Rationale:** The main gap is guidance for review and post-incident reasoning,
not missing runtime primitives. A guide and template improve review without
pretending the repo can automate architectural judgment.

**Alternative:** Start with resilience-specific runtime utilities. Rejected
because public guidance should land before any framework-level opinion.

### 2. Keep the guide ordered around symptom-to-design reasoning

**Choice:** The guide keeps these exact second-level headings, each exactly once
and in the listed order, with no extra second-level headings:
- `## Incident, symptom, and immediate workaround`
- `## Five whys to the design decision`
- `## Boundary, blast radius, and affected invariants`
- `## Retry, replay, idempotence, and degradation`
- `## Fail-open vs fail-closed`
- `## Reviewer prompts and follow-ups`

Section ownership stays explicit:
- `## Incident, symptom, and immediate workaround` owns the observed failure
  and any short-term operator action.
- `## Five whys to the design decision` owns the climb from symptom to design
  cause.
- `## Boundary, blast radius, and affected invariants` owns scope,
  containment, and what system guarantees were threatened.
- `## Retry, replay, idempotence, and degradation` owns repeat-delivery,
  replay, partial-failure, and degraded-service reasoning.
- `## Fail-open vs fail-closed` owns the chosen failure posture and why.
- `## Reviewer prompts and follow-ups` owns explicit review questions,
  ownership, and the rule that restart-or-retry alone is not root cause.

Under each heading, the guide includes at least one explicit reviewer prompt or
checklist question.

**Rationale:** The change should make resilience review concrete and ordered.
Exact headings make the doctrine reviewable and keep deterministic proof strong.

**Alternative:** Publish a broad reliability essay. Rejected because it would be
harder to verify and less useful as a review kit.

### 3. Keep the template fill-in only and aligned to the guide

**Choice:** The template keeps these exact second-level headings, each exactly
once and in the listed order, with no extra second-level headings:
- `## Incident or scenario under review`
- `## Symptom and immediate workaround`
- `## Five whys chain`
- `## Boundary, blast radius, and affected invariants`
- `## Retry, replay, idempotence, and degradation assumptions`
- `## Fail-open vs fail-closed decision`
- `## Follow-ups, owners, and non-goals`

Every template section stays a fill-in field or prompt rather than free-form
narrative prose.

**Rationale:** Maintainers need a reusable artifact shape for postmortems,
review notes, and design checks.

### 4. Link the guide and template from public entry points only

**Choice:** Update `README.md` to add these exact Markdown links:
- `[Resilience review guide](docs/resilience-review.md)`
- `[Five-whys architecture template](templates/five-whys-architecture.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links:
- `[Resilience review guide](resilience-review.md)`
- `[Five-whys architecture template](../templates/five-whys-architecture.md)`

This change does not add a contributor-doc link to a canonical main resilience
spec because no such synced main spec exists yet.

**Rationale:** Public entry points should surface the guide and template without
creating broken links to a future main spec path.

### 5. Require a saved human review record for resilience quality

**Choice:** Save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`.
That file uses these exact labeled lines:
- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/resilience-review.md, templates/five-whys-architecture.md`
- `question 1: <pass|fail> - guide climbs from symptom and workaround to a design decision rather than stopping at retry or restart`
- `question 2: <pass|fail> - guide makes blast radius, affected invariants, retry or replay assumptions, and degradation posture explicit`
- `question 3: <pass|fail> - guide requires an explicit fail-open versus fail-closed decision, follow-up ownership, and the rule that restart or retry alone is not root cause`
- `question 4: <pass|fail> - every required guide heading includes at least one explicit reviewer prompt or checklist question`
- `question 5: <pass|fail> - template covers every required section and each section stays a fill-in prompt rather than free-form prose`
- `verdict: <pass|fail>`

For acceptance, questions 1 through 5 must all be `pass` and `verdict` must be
`pass`.

**Rationale:** Exact headings and links do not prove the guide actually teaches
resilience review well.

### 6. Save deterministic proof artifacts with exact output contracts

**Choice:** Deterministic verification saves these exact artifacts under
`openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/`:
- `resilience-guide-headings.txt` prints the full detected list of guide
  second-level headings, one per line, after validation confirms the list is an
  exact no-extra match.
- `five-whys-template-headings.txt` prints the full detected list of template
  second-level headings, one per line, after validation confirms the list is an
  exact no-extra match.
- `readme-resilience-links.txt` prints these exact lines in order:
  - `exists:docs/resilience-review.md`
  - `exists:templates/five-whys-architecture.md`
  - `[Resilience review guide](docs/resilience-review.md)`
  - `[Five-whys architecture template](templates/five-whys-architecture.md)`
- `contributor-resilience-links.txt` prints these exact lines in order:
  - `exists:docs/resilience-review.md`
  - `exists:templates/five-whys-architecture.md`
  - `[Resilience review guide](resilience-review.md)`
  - `[Five-whys architecture template](../templates/five-whys-architecture.md)`
- `human-resilience-review-labels.txt` prints these exact lines in order:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/resilience-review.md, templates/five-whys-architecture.md:1`
  - `question 1: pass - guide climbs from symptom and workaround to a design decision rather than stopping at retry or restart:1`
  - `question 2: pass - guide makes blast radius, affected invariants, retry or replay assumptions, and degradation posture explicit:1`
  - `question 3: pass - guide requires an explicit fail-open versus fail-closed decision, follow-up ownership, and the rule that restart or retry alone is not root cause:1`
  - `question 4: pass - every required guide heading includes at least one explicit reviewer prompt or checklist question:1`
  - `question 5: pass - template covers every required section and each section stays a fill-in prompt rather than free-form prose:1`
  - `verdict: pass:1`

**Rationale:** Exact artifacts make review independent of summarized prose.

## Risks / Trade-offs

**[Resilience language may overlap with safety docs]** -> Readers may confuse
this guide with a broader safety case. Mitigation: keep the guide centered on
symptom-to-design reasoning and failure posture rather than certification.

**[No canonical main resilience spec yet]** -> Contributors may expect a main
spec link immediately. Mitigation: keep docs linked only to guide/template until
a canonical spec exists through sync/archive.

**[Scope creep into incident tooling]** -> Reviewers may ask for postmortem or
chaos automation in the same change. Mitigation: keep tooling explicitly out of
scope.
