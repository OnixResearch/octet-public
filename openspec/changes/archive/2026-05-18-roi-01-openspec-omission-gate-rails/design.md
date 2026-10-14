## Context

Prior Octet changes hardened individual OpenSpec template contracts, but review
metrics still show omission findings as the largest repeated failure class:
design prompt omissions repeat 80 times, spec prompt omissions 41 times,
proposal prompt omissions 33 times, and task-level human/auto-fix omission
findings repeat 23 and 16 times. The repeated examples are not arbitrary prose
style issues; they are missing scenario coverage, boundary cases, proof signals,
verification expectations, and self-verifiable task inputs.

## Goals / Non-Goals

**Goals:**
- turn promoted omission findings into compact stage-template prompts
- keep existing template heading inventories and required lines intact
- make config rules and deterministic evidence prove the prompts stay visible
- keep implementation, verification, and human/oracle tasks split

**Non-Goals:**
- changing product lints or CLI behavior
- replacing reviewer judgment with a purely mechanical checker
- rewriting archived changes

## Decisions

### 1. Add omission-prevention prompts without changing template heading contracts

**Choice:** add checklist-style prompt text inside the existing proposal,
delta-spec, design, and tasks templates, while preserving current required
second-level heading order and exact required lines.

**Covers:** `Scenario: Maintainer sees repeated proposal omission checks before drafting`; `Scenario: Maintainer sees repeated delta-spec omission checks before review`; `Scenario: Maintainer sees repeated design omission checks before implementation`; `Scenario: Maintainer sees repeated task omission checks before gate review`

**Rationale:** the repeated findings show authors need visible prompts at the
moment they draft each artifact, but changing heading inventories would create
avoidable churn against existing specs.

**Alternative considered:** add a separate long-form guide. Rejected because
maintainers copy the templates during active work; a separate guide is easier to
miss and harder to gate deterministically.

**Implementation:** patch `openspec/templates/*.md` with a short
"Omission-prevention checklist" block per stage. Keep required heading lines,
required comments, and required example lines unchanged. For task guidance, put
reminders near the existing task ID rules and `H<n>` field layout.

**Boundary cases:** the prompt must not require irrelevant negative controls for
simple documentation-only changes; it should say to name boundary cases when the
requirement has non-trigger, continued-warning, or continued-blocking behavior.

### 2. Mirror the omission prompts in `openspec/config.yaml`

**Choice:** extend repo-local proposal/design/tasks rules so gate prompts repeat
the same promoted omission checks as the templates.

**Covers:** `Scenario: Config guidance includes promoted omission-prevention checks`; `Scenario: Config task guidance includes promoted task omission checks`; `Scenario: Human checkpoint guidance blocks implicit oracle inputs`

**Rationale:** template text alone helps humans, but gate prompts need the same
criteria so reviewers flag the same gaps consistently.

**Alternative considered:** rely only on review_metrics summaries. Rejected
because metrics are retrospective; config guidance is available before review.

**Implementation:** patch `openspec/config.yaml` rules with compact clauses for
proposal verification expectations, delta-spec boundary/negative cases, design
coverage/proof signals, and task human/oracle replay fields.

**Boundary cases:** keep existing exact-finding text for task status gate
blockers unchanged so deterministic tests do not lose their stable messages.

### 3. Verify with deterministic extracts and gate transcripts

**Choice:** create a small evidence helper under the change evidence directory
that extracts the required prompt lines from templates/config and writes stable
artifacts; then run `openspec validate` and stage gates.

**Covers:** `Scenario: Maintainer sees repeated proposal omission checks before drafting`; `Scenario: Maintainer sees repeated delta-spec omission checks before review`; `Scenario: Maintainer sees repeated design omission checks before implementation`; `Scenario: Config guidance includes promoted omission-prevention checks`; `Scenario: Config task guidance includes promoted task omission checks`; `Scenario: Maintainer sees repeated task omission checks before gate review`; `Scenario: Human checkpoint guidance blocks implicit oracle inputs`

**Rationale:** reviewers need inspectable proof that the prompts exist and that
existing OpenSpec contracts still validate.

**Alternative considered:** cite only diffs. Rejected because this repo prefers
saved deterministic artifacts for OpenSpec evidence.

**Implementation:** add `evidence/update-omission-artifacts.py` that prints and
asserts exact snippets for each stage plus preserved template contracts: proposal/design heading order, spec guidance comments, and task status layout. Save outputs under the active change's
`evidence/` directory, then archive paths will be rewritten naturally by
OpenSpec archive if the change is completed later.

**Boundary cases:** if live `openspec_gate` execution is unavailable, save the
blocker transcript and keep deterministic extracts plus `openspec validate` as
the non-oracle proof; do not mark the H-task complete without a gate transcript.

## Exact Prompt And Config Snippets

The implementation ships these exact new omission-prevention snippets:

- proposal template: `<!-- Omission-prevention checklist: name planned verification/evidence expectations, exact affected files/artifacts, and explicit non-goal boundaries before writing specs or tasks. -->`
- delta-spec template: `<!-- Omission-prevention checklist: enumerate boundary cases, negative controls, and continued-warning or continued-blocking scenarios when the requirement has non-trigger or regression-catching behavior. -->`
- design template: `<!-- Omission-prevention checklist: check every added/modified scenario heading and every zero-scenario requirement against decision coverage, boundary cases, continued-warning cases, and exact proof acceptance signals before implementation starts. -->`
- tasks template: `<!-- Omission-prevention checklist: keep implementation separate from verification/evidence, give every added or modified scenario its own coverage row, and make each H<n> oracle checkpoint self-verifiable with exact command=, input=, asserts=, and evidence= fields. -->`
- config proposal rule: `Follow openspec/templates/proposal.md; name exact repo-relative files/artifacts, ordering/dependencies, explicit non-goal boundaries, and planned saved evidence or helper/gate runs before drafting specs or tasks`
- config delta-spec rule: `Follow openspec/templates/spec.md ... enumerate boundary cases/negative controls/continued-warning or continued-blocking cases ... and describe exact proof output/schema when artifacts matter`
- config design rule: `Follow openspec/templates/design.md ... every added or modified scenario heading or zero-scenario requirement must appear in a decision's **Covers:** field ... every saved proof artifact must state its exact expected output or acceptance signal`
- config tasks rules: keep implementation, deterministic verification, and human/oracle checkpoints separate; keep one verification coverage row per scenario; require exact H-task `command=`, `input=`, `asserts=`, and `evidence=` fields.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Maintainer sees repeated proposal omission checks before drafting`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/proposal-template-omission-prompts.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`
- exact expected output / acceptance signal:
  - artifact prints the proposal omission-prevention checklist lines mentioning verification/evidence expectations, affected files/artifacts, and non-goal boundaries
- ordering / prerequisites:
  - patch `openspec/templates/proposal.md` first
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - `Scenario: Maintainer sees repeated delta-spec omission checks before review`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/spec-template-omission-prompts.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`
- exact expected output / acceptance signal:
  - artifact prints spec-template lines mentioning boundary cases, negative controls, continued-warning or continued-blocking cases, and exact proof output/schema
- ordering / prerequisites:
  - patch `openspec/templates/spec.md` first
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - `Scenario: Maintainer sees repeated design omission checks before implementation`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/design-template-omission-prompts.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`
- exact expected output / acceptance signal:
  - artifact prints design-template lines mentioning scenario coverage, boundary cases, continued-warning cases, and exact proof acceptance signals
- ordering / prerequisites:
  - patch `openspec/templates/design.md` first
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - `Scenario: Config guidance includes promoted omission-prevention checks`
  - `Scenario: Config task guidance includes promoted task omission checks`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`
- exact expected output / acceptance signal:
  - artifact prints final `openspec/config.yaml` rule lines that mention proposal evidence/artifact expectations, spec boundary/negative cases, design coverage/proof signals, task split/coverage obligations, and task oracle replay fields
- ordering / prerequisites:
  - patch `openspec/config.yaml` after template wording is final
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - `Scenario: Maintainer sees repeated task omission checks before gate review`
  - `Scenario: Human checkpoint guidance blocks implicit oracle inputs`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`
- exact expected output / acceptance signal:
  - artifact prints task-template lines mentioning split I/V/H units, one scenario per coverage row, and exact H-task `command=`, `input=`, `asserts=`, `evidence=` fields
- ordering / prerequisites:
  - patch `openspec/templates/tasks.md` first
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - repeated review-metrics omission source evidence
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/review-metrics-omissions.txt`
- command / helper:
  - `review_metrics_promotions --cwd /path/to/git/octet --min 2 --limit 5`
- exact expected output / acceptance signal:
  - artifact lists repeated omission classes for proposal/spec/design/tasks with counts that motivate the promoted prompts
- ordering / prerequisites:
  - capture after current review metrics are available
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - active change validity
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/openspec-validate.txt`
- command / helper:
  - `openspec validate roi-01-openspec-omission-gate-rails --strict --json`
- exact expected output / acceptance signal:
  - artifact shows the active change is valid with zero issues
- ordering / prerequisites:
  - run after template, config, task, and spec updates
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - proposal/design/tasks stage gate oracle review
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/proposal-gate.txt`
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/design-gate.txt`
  - `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-gate.txt`
- command / helper:
  - `openspec_gate stage=proposal change=roi-01-openspec-omission-gate-rails`
  - `openspec_gate stage=design change=roi-01-openspec-omission-gate-rails`
  - `openspec_gate stage=tasks change=roi-01-openspec-omission-gate-rails`
- exact expected output / acceptance signal:
  - each artifact includes a passing verdict or records an explicit blocker transcript that keeps the corresponding H-task open
- ordering / prerequisites:
  - run after deterministic artifacts and validation pass
- cleanup / postcondition:
  - no temp files remain

## Risks / Trade-offs

- Template noise could slow small changes -> keep prompts short and scoped to repeated omission classes.
- Prompt-only hardening may not catch every issue -> pair prompts with config guidance and saved extracts, then promote deterministic blockers only if repetition persists.
- Active-change evidence paths later move on archive -> save active paths in tasks now and rewrite/archive through the normal OpenSpec lifecycle.
