## Context

The current design template already asks decisions to include `**Covers:**` and
`**Boundary cases:**`, but review metrics still show 80 repeated
`omission|design|prompt` findings. The common gap is not merely missing a
scenario name; it is that proposal promises, spec boundary obligations, or proof
expectations fail to reappear in `design.md` as explicit decisions or proof-plan
entries.

## Goals / Non-Goals

**Goals:**
- make proposal/spec obligation carry-forward visible in the design template
- preserve a blocked fixture where one proposal/spec obligation is intentionally missing from design coverage
- preserve a passing fixture where the same obligation is explicitly mapped

**Non-Goals:**
- changing the task-stage governance contract
- adding new Rust lint behavior
- rewriting archived OpenSpec designs

## Decisions

### 1. Design templates include an obligation cross-check

**Choice:** add `**Obligation cross-check:**` to the decision block and add the same phrase to the opening omission-prevention comment.

**Covers:** `Scenario: Maintainer sees repeated design omission checks before implementation`; `Scenario: Design author cross-checks proposal and spec obligations`

**Obligation cross-check:** carries forward the proposal promise to block repeated design omissions, the delta-spec scenario that requires proposal/spec obligations to map into design, the config-guidance scenario's proposal-rule and delta-spec-rule bullets, and the verification expectation for a saved fixture pair.

**Rationale:** repeated design omissions have survived scenario-only `**Covers:**` prompts. A named cross-check line forces authors and reviewers to ask whether proposal verification promises, non-goal boundaries, delta-spec negative controls, and proof artifacts are represented in the design.

**Alternative considered:** keep expanding `**Covers:**` prose. Rejected because `**Covers:**` is already present and the promoted findings show reviewers still see obligations omitted.

**Implementation:** update `openspec/templates/design.md` and `openspec/config.yaml` with explicit proposal/spec obligation mapping language, then sync the accepted `openspec-stage-authoring` spec with the same contract after validation.

**Boundary cases:** the cross-check is a design-review prompt, not a parser-enforced schema; the human/oracle fixture gate remains responsible for checking semantic omissions in design text.

### 2. Checked-in fixtures capture missing and covered obligation cases

**Choice:** add `tests/fixtures/openspec-design-governance/design-obligation-missing-blocked` and `design-obligation-covered-pass`.

**Covers:** `Scenario: Design author cross-checks proposal and spec obligations`; `Scenario: Config guidance includes promoted omission-prevention checks`

**Obligation cross-check:** the blocked fixture intentionally promises a `same-branch guard suppression` obligation in proposal/spec text but omits it from design decisions and proof planning; the pass fixture names that obligation in `**Covers:**`, `**Obligation cross-check:**`, `**Boundary cases:**`, and the deterministic proof plan.

**Rationale:** templates are easy to regress. A small fixture pair gives future agents a stable before/after target when rerunning the design gate or adjusting prompt wording.

**Alternative considered:** rely only on the active change's own gate transcript. Rejected because a reusable fixture makes the promoted omission class reproducible after this change is archived.

**Implementation:** add fixture files plus a `run-design-fixture-gate.sh` helper that temporarily copies a fixture to `openspec/changes/<fixture>` and invokes `openspec_gate stage=design` through the repo-local extension.

**Boundary cases:** fixture transcripts are human/oracle evidence; deterministic verification only checks fixture shape and the presence/absence of the intended obligation text.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Maintainer sees repeated design omission checks before implementation`
  - `Scenario: Design author cross-checks proposal and spec obligations`
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-template-obligation-fields.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/check-design-obligation-fixtures.py --template`
- exact expected output / acceptance signal:
  - the output lists the design-template omission-prevention comment and the `**Obligation cross-check:**` decision label exactly once
- ordering / prerequisites:
  - run after updating `openspec/templates/design.md`
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - `Scenario: Design author cross-checks proposal and spec obligations`
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-fixtures.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/check-design-obligation-fixtures.py --fixtures`
- exact expected output / acceptance signal:
  - the output reports the missing fixture lacks `same-branch guard suppression` in `design.md` while the pass fixture includes it in `**Covers:**`, `**Obligation cross-check:**`, `**Boundary cases:**`, and the proof plan
- ordering / prerequisites:
  - run after adding both checked-in fixture directories
- cleanup / postcondition:
  - no active fixture copy remains under `openspec/changes/`

- scenario or requirement:
  - `Scenario: Config guidance includes promoted omission-prevention checks`
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-stage-guidance.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/check-design-obligation-fixtures.py --config`
- exact expected output / acceptance signal:
  - the output prints the proposal, delta-spec, and design guidance rules; the design rule includes `proposal/spec obligation cross-checks`, while the proposal rule includes verification/evidence expectations and exact affected artifacts, and the delta-spec rule includes boundary cases, negative controls, and exact proof output expectations
- ordering / prerequisites:
  - run after updating `openspec/config.yaml`
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - change-level OpenSpec validation promised by proposal Testing
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/openspec-validate.txt`
- command / helper:
  - `openspec validate harden-design-obligation-gate --strict --json`
- exact expected output / acceptance signal:
  - JSON summary reports one `change` item for `harden-design-obligation-gate` with `valid: true`, zero issues, and one passed item
- ordering / prerequisites:
  - run after proposal, delta spec, design, tasks, fixtures, and helper updates are in place
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - patch-hygiene verification promised by proposal Testing
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/git-diff-check.txt`
- command / helper:
  - `git diff --check`
- exact expected output / acceptance signal:
  - command exits 0 and prints no whitespace-error lines
- ordering / prerequisites:
  - run after all file edits and generated evidence updates
- cleanup / postcondition:
  - working tree may remain dirty only with intentional change files

- scenario or requirement:
  - human/oracle checkpoint for the missing fixture
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-missing-gate.txt`
- command / helper:
  - `tests/fixtures/openspec-design-governance/run-design-fixture-gate.sh design-obligation-missing-blocked`
- exact expected output / acceptance signal:
  - the transcript reports a design-stage blocker for the intentionally missing same-branch guard suppression obligation
- ordering / prerequisites:
  - run after deterministic fixture-shape checks pass
- cleanup / postcondition:
  - helper removes the temporary active change copy

- scenario or requirement:
  - human/oracle checkpoint for the covered fixture
- artifact path:
  - `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-covered-gate.txt`
- command / helper:
  - `tests/fixtures/openspec-design-governance/run-design-fixture-gate.sh design-obligation-covered-pass`
- exact expected output / acceptance signal:
  - the transcript reports a passing design-stage gate result for the fixture with no blocker findings about missing same-branch guard suppression mapping
- ordering / prerequisites:
  - run after deterministic fixture-shape checks pass
- cleanup / postcondition:
  - helper removes the temporary active change copy

## Risks / Trade-offs

- Human/oracle gate wording can drift -> keep deterministic fixture-shape checks so the intended omission remains explicit even if a later reviewer prompt changes.
- More template fields can feel heavy -> limit the addition to one cross-check line that explains why proposal/spec obligations cannot disappear.
