# Design: Clarify Tiger Style Safety-Critical Claims

## Context

Tiger Style already speaks in the language of safety, proofs, reviewable
artifacts, and fail-fast design. That is useful because the repo really does
care about high-consequence software. But public docs still leave too much room
for readers to treat lint cleanliness, test coverage, or a good review note as
if they were a complete safety argument. The missing piece is not another lint.
It is one public guide that draws a sharp boundary around supported claims.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for safety-critical claim boundaries.
- Make the guide discoverable from the README and contributor docs.
- Keep the wording aligned with the existing main safety spec instead of
  inventing a second safety philosophy.
- Give maintainers explicit wording rules for docs, reviews, and rollout notes.

**Non-Goals:**
- New safety lints or stronger lint severities.
- Certification, legal, or regulator-submission workflow.
- A complete safety case template for downstream products.
- Replacing the existing safety spec with prose-only guidance.

## Decisions

### 1. Use a public claims guide, not another enforcement surface

**Choice:** Add `docs/safety-critical.md` as a public claims-boundary guide.

**Rationale:** The core problem is overstatement, not missing syntax checks.
The repo needs a human-readable statement of what Tiger Style can and cannot
support.

**Alternative:** Add a lint that forbids phrases like "safety-critical" or
"verified". Rejected because the real issue is contextual meaning, not a small
set of banned tokens.

### 2. Make the guide deterministic and section-owned

**Choice:** The guide keeps these exact second-level headings, each exactly once
and in the listed order, with no extra second-level headings:
- `## What Tiger Style can claim`
- `## What Tiger Style cannot claim`
- `## Lints, tests, proof, assurance, and certification`
- `## Evidence and human review`
- `## Responsible wording in docs and reviews`

Under those headings, the guide MUST cover these exact responsibilities:
- `## What Tiger Style can claim` owns supported claims such as linted policy,
  reviewable guidance, and explicit evidence expectations.
- `## What Tiger Style cannot claim` owns the rule that lint cleanliness,
  passing tests, or a finished review kit do not by themselves imply
  certification, regulator approval, or a complete safety case.
- `## Lints, tests, proof, assurance, and certification` owns the difference
  between those surfaces and says none of them should be conflated.
- `## Evidence and human review` owns the requirement that high-consequence
  claims need named evidence and human judgment beyond green automation.
- `## Responsible wording in docs and reviews` owns concrete preferred/avoid
  phrasing for README text, rollout notes, and review summaries.

Each heading includes at least one explicit maintainer prompt or wording rule.

**Rationale:** This change is mostly about exact public wording. Exact headings
keep deterministic verification strong enough to prove the guide did not drift
into vaguer or more promotional language.

**Alternative:** Publish a shorter FAQ. Rejected because it would weaken
traceability and make review drift harder to catch.

### 3. Put discoverability in the documented entry points

**Choice:** Update `README.md` to add this exact Markdown link:
- `[Safety-critical claims guide](docs/safety-critical.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links:
- `[Safety-critical claims guide](safety-critical.md)`
- `[Safety spec contract](../openspec/specs/safety/spec.md)`

**Rationale:** Readers should be able to find the claims guide from the same
entry points they already use for lint catalogs and contributor touchpoints.

**Alternative:** Link only from the new guide. Rejected because that keeps the
material effectively hidden.

### 4. Require a saved human review record for claim language

**Choice:** Save one human/oracle review record at
`openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`.
That file uses these exact labeled lines:
- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/safety-critical.md`
- `question 1: <pass|fail> - guide distinguishes clearly between supported Tiger Style claims and unsupported certification or assurance claims`
- `question 2: <pass|fail> - guide explains the difference among lints, tests, formal proof, assurance evidence, and certification without overstating any one surface`
- `question 3: <pass|fail> - guide says high-consequence claims need named evidence and human review beyond green automation`
- `question 4: <pass|fail> - every required section includes at least one explicit maintainer prompt or wording rule`
- `question 5: <pass|fail> - guide gives explicit wording guidance for docs, reviews, and rollout notes so maintainers do not imply unsupported safety-critical guarantees`
- `verdict: <pass|fail>`

For acceptance, questions 1 through 5 must all be `pass` and `verdict` must be
`pass`.

**Rationale:** Deterministic checks can prove headings and links. They cannot
prove that nuanced claims language is accurate and responsible.

**Alternative:** Use deterministic checks only. Rejected because the risk here
is semantic overstatement.

### 5. Save deterministic proof artifacts with exact output contracts

**Choice:** Deterministic verification saves these exact artifacts under
`openspec/changes/clarify-safety-critical-claims/evidence/`:
- `safety-critical-guide-headings.txt` prints the full detected list of guide
  second-level headings, one per line, after validation confirms the list is an
  exact no-extra match.
- `readme-safety-critical-links.txt` prints these exact lines in order:
  - `exists:docs/safety-critical.md`
  - `[Safety-critical claims guide](docs/safety-critical.md)`
- `contributor-safety-critical-links.txt` prints these exact lines in order:
  - `exists:docs/safety-critical.md`
  - `exists:openspec/specs/safety/spec.md`
  - `[Safety-critical claims guide](safety-critical.md)`
  - `[Safety spec contract](../openspec/specs/safety/spec.md)`
- `human-safety-critical-review-labels.txt` prints these exact lines in order:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/safety-critical.md:1`
  - `question 1: pass - guide distinguishes clearly between supported Tiger Style claims and unsupported certification or assurance claims:1`
  - `question 2: pass - guide explains the difference among lints, tests, formal proof, assurance evidence, and certification without overstating any one surface:1`
  - `question 3: pass - guide says high-consequence claims need named evidence and human review beyond green automation:1`
  - `question 4: pass - every required section includes at least one explicit maintainer prompt or wording rule:1`
  - `question 5: pass - guide gives explicit wording guidance for docs, reviews, and rollout notes so maintainers do not imply unsupported safety-critical guarantees:1`
  - `verdict: pass:1`

**Rationale:** Exact artifact contracts make later review independent of prose
summaries.

## Risks / Trade-offs

**[Public caution may read as weakness]** -> Some readers may see the guide as
backing away from strong safety posture. Mitigation: keep `## What Tiger Style
can claim` concrete before drawing unsupported-claim boundaries.

**[Glossary drift]** -> Terms like "assurance" or "certification" can drift over
time. Mitigation: keep the contributor doc linked to the canonical main safety
spec and require human review on wording quality.

**[Scope creep into legal guidance]** -> Reviewers may ask for regulator or
standards mapping in the same change. Mitigation: keep legal and certification
workflow explicitly out of scope.
