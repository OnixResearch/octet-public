## Context

The design and delta-spec stages now have explicit obligation cross-checks, but review metrics still report `omission|proposal|prompt` findings. The missing information is often only knowable at proposal time: exact evidence expectations, affected artifacts, downstream scope, non-goals, and package/profile membership.

## Goals / Non-Goals

**Goals:** make proposal omissions visible at proposal-gate time; add deterministic negative/positive fixture coverage; preserve the archive/evidence closeout pattern used by the prior hardening slices.

**Non-Goals:** runtime lint changes, object corpus changes, historical archive rewrites, or solving the tasks/human omission clusters in this slice.

## Decisions

### 1. Proposal template carries explicit obligation fields

**Choice:** Expand `openspec/templates/proposal.md` with concrete proposal prompts for exact affected artifacts, concrete scope/package/profile membership, downstream validation boundaries, explicit non-goals, and saved verification/evidence expectations.

**Covers:** `Proposal Template Captures Scope, Artifacts, And Ordering`; `Proposal governance fixtures prove blocked and covered obligations`.

**Obligation cross-check:** Carries the proposal promise that this slice blocks missing verification/evidence expectations, exact affected artifacts, concrete package/profile scope, downstream validation boundaries, and non-goal boundaries. The prompt text must be visible before specs/design/tasks are drafted.

**Rationale:** Proposal-stage omissions cannot be fully repaired by later spec/design prompts because the missing scope/evidence facts may never be stated.

**Boundary cases:** A proposal that only says “add tests” or “update docs” without exact evidence/artifact/scope details remains under-specified. A proposal that explicitly names the affected paths, proof commands/artifacts, scoped packages/profiles, and excluded work should pass the proposal fixture.

**Proof plan:** Static helper evidence must print `PASS proposal template includes obligation fields`. The missing proposal fixture must be blocked by the proposal gate; the covered fixture must pass.

### 2. Config proposal rule mirrors the template contract

**Choice:** Update `openspec/config.yaml` proposal rules so gate prompts ask reviewers to check the same proposal obligations: evidence, artifacts, concrete scope, downstream boundaries, and non-goals.

**Covers:** `Repo Guidance Points Authors At Stage Templates`; `Proposal governance fixtures prove blocked and covered obligations`.

**Obligation cross-check:** Maps the proposal-stage omitted-field problem into reviewer-visible config guidance before delta-spec and design review.

**Rationale:** Gates consume config guidance; a template-only fix is easy to miss when reviewing existing changes that do not freshly copy the template.

**Boundary cases:** The config rule must not weaken existing delta-spec/design guidance or remove existing omission-prevention checks.

**Proof plan:** Static helper evidence must print `PASS config includes proposal-stage obligation guidance`; `openspec validate --all --strict --json` must pass after archive.

### 3. Fixture pair proves missing and covered proposal obligations

**Choice:** Add `tests/fixtures/openspec-proposal-governance/proposal-obligation-missing-blocked` and `proposal-obligation-covered-pass` plus a runner that calls the proposal gate.

**Covers:** `Proposal governance fixtures prove blocked and covered obligations`.

**Obligation cross-check:** The negative fixture omits exact verification/artifact/scope obligations while the spec/design rely on them; the positive fixture names those obligations explicitly in proposal Impact/Non-Goals and carries them into spec/design.

**Rationale:** Deterministic fixture evidence prevents this from becoming prose-only guidance.

**Boundary cases:** The blocked oracle must require a proposal/omission/missing finding rather than any unrelated gate failure. The passing oracle must reject `VERDICT: FAIL` or `blocked`.

**Proof plan:** Expected outputs are `PASS expected blocked fixture: proposal-obligation-missing-blocked` and `PASS expected passing fixture: proposal-obligation-covered-pass`.

## Risks / Trade-offs

- **Over-broad proposal burden** → Keep prompts focused on recurring omission classes and exact artifacts/proof outputs.
- **False-positive fixture success** → Require target proposal/omission/missing text for the blocked fixture.
- **Archive path drift** → Rerun the helper from the archive path and rewrite active paths after `openspec archive`.

## Deterministic Proof Plan

- `python openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/check-proposal-obligation-fixtures.py template` → `PASS proposal template includes obligation fields`.
- `python openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/check-proposal-obligation-fixtures.py config` → `PASS config includes proposal-stage obligation guidance`.
- `python openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/check-proposal-obligation-fixtures.py fixtures` → `PASS proposal-obligation fixtures encode missing and covered obligations`.
- `tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh proposal-obligation-missing-blocked` → `PASS expected blocked fixture: proposal-obligation-missing-blocked`.
- `tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh proposal-obligation-covered-pass` → `PASS expected passing fixture: proposal-obligation-covered-pass`.
- Proposal, design, and tasks gates for this change pass; final `openspec validate --all --strict --json` and `git diff --check` pass after archive.
