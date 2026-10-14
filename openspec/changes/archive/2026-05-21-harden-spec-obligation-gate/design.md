## Context

The latest Octet governance hardening added design-stage obligation cross-checks, but review metrics still show `omission|spec|prompt` as the next largest recurring class. The existing spec template already asks for boundary/negative/continued-warning scenarios, but it does not explicitly force proposal and design obligations into delta-spec requirements/scenarios.

## Goals / Non-Goals

**Goals:**
- Make the delta-spec template and config prompt authors/reviewers to cross-check proposal/design obligations.
- Prove the prompt with a failing fixture where a proposal promise is absent from the delta spec.
- Prove the positive path with a fixture where the obligation is encoded as a named scenario.

**Non-Goals:**
- No external gate-extension rewrite.
- No broad archive normalization.

## Decisions

### 1. Add a spec-template obligation cross-check comment

**Choice:** Extend `openspec/templates/spec.md` with an obligation cross-check comment covering proposal promises, affected artifacts, non-goal boundaries, and design decision/proof obligations.

**Covers:** `Scenario: Spec author cross-checks proposal and design obligations`

**Obligation cross-check:** Carries the proposal's verification/evidence promise and non-goal boundary into an explicit delta-spec authoring prompt before proposal-stage gate review.

**Rationale:** The recurring finding is not only missing boundary examples; it is missing carry-forward from earlier artifacts into normative spec text.

**Alternative considered:** Rely on the existing boundary-case comment. Rejected because metrics still show proposal/design promises disappearing from specs.

**Implementation:** Patch `openspec/templates/spec.md` with the exact `<!-- Obligation cross-check: map every proposal verification/evidence promise, affected artifact, non-goal boundary, and design decision/proof obligation into an explicit delta-spec requirement or named scenario before proposal-stage gate review. -->` comment. Patch the existing `openspec/config.yaml` proposal rule for `openspec/templates/spec.md` so its contract includes: `cross-check proposal/design obligations into explicit requirements or named scenarios`, `boundary cases/negative controls/continued-warning or continued-blocking cases`, and `exact proof output/schema`.

**Boundary cases:** The prompt must include proposal obligations and design obligations, not only generic boundary cases.

### 2. Add positive and negative spec-governance fixtures

**Choice:** Add `tests/fixtures/openspec-spec-governance/spec-obligation-missing-blocked` and `spec-obligation-covered-pass`, plus a runner that invokes the proposal gate for either fixture.

**Covers:** `Scenario: Config guidance includes promoted omission-prevention checks`, `Scenario: Maintainer sees repeated delta-spec omission checks before review`

**Obligation cross-check:** The missing fixture proposal requires a non-goal boundary/evidence behavior that the spec omits; the covered fixture encodes the same class of obligation as a named scenario.

**Rationale:** Fixtures keep the rule reviewable and prevent prose-only hardening.

**Alternative considered:** Only add static text checks. Rejected because the promoted review finding is a stage-gate omission class.

**Implementation:** Copy fixtures into temporary active changes and call `openspec_gate stage=proposal` through the existing Pi extension. The negative fixture is expected to report a spec omission; the runner accepts it only when the gate exits nonzero or emits a failing/blocked finding whose text includes `omission` plus `spec` or `missing`. The positive fixture must exit zero and avoid `VERDICT: FAIL`/`blocked`.

**Boundary cases:** The failing fixture must fail because of a missing spec obligation, while the passing fixture must name the obligation in the delta spec.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Spec author cross-checks proposal and design obligations`
  - `Scenario: Config guidance includes promoted omission-prevention checks`
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-template-obligation-fields.txt`
  - `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-stage-guidance.txt`
  - `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-fixtures.txt`
  - `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-missing-gate.txt`
  - `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-covered-gate.txt`
- command / helper:
  - `python openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/check-spec-obligation-fixtures.py --check template`
  - `python openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/check-spec-obligation-fixtures.py --check config`
  - `python openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/check-spec-obligation-fixtures.py --check fixtures`
  - `tests/fixtures/openspec-spec-governance/run-spec-fixture-gate.sh spec-obligation-missing-blocked`
  - `tests/fixtures/openspec-spec-governance/run-spec-fixture-gate.sh spec-obligation-covered-pass`
- exact expected output / acceptance signal:
  - `PASS template includes spec obligation cross-check fields`
  - `PASS config includes spec-stage proposal/design obligation guidance`
  - `PASS spec-obligation fixtures include missing and covered cases`
  - missing fixture transcript contains `PASS expected blocked fixture: spec-obligation-missing-blocked` after a proposal-gate output containing `omission` and either `spec` or `missing`
  - covered fixture transcript contains `PASS expected passing fixture: spec-obligation-covered-pass` and no `VERDICT: FAIL` or `blocked` finding
- ordering / prerequisites:
  - template/config/fixtures exist before gate runs
- cleanup / postcondition:
  - fixture runner removes temporary active changes

## Risks / Trade-offs

- Gate wording may be reviewer-model-sensitive -> mitigate with static text helper checks and saved transcripts.
- A broad prompt could create noisy findings -> scope wording to proposal/design obligations that are verification, artifact, boundary, decision, proof, negative-control, or continued-warning claims.
