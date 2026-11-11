# Design: Add Tiger Style Performance Review Kit

## Context

The repo already documents Tiger Style performance principles in the agent skill and in `openspec/specs/performance/spec.md`, but the public docs exposed from `README.md` still focus on lints, rollout, and contributor touchpoints. There is no stable public page that turns Tiger Style performance rules into a repeatable checklist, and there is no copyable template for recording napkin math, cost × frequency reasoning, batching, or bounds.

A compile-time lint pass is the wrong first move here. Most Tiger Style performance guidance is design-time and workload-dependent. Static detection would either miss the important cases or over-warn on code that is structurally similar but operationally harmless.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for Tiger Style performance review.
- Publish one copyable template for performance sketches and napkin math.
- Make both artifacts discoverable from the README and contributor docs.
- Keep the content aligned with the existing Tiger Style performance spec instead of inventing a second performance philosophy.

**Non-Goals:**
- New AST lints for performance rules.
- Benchmark automation, CI perf budgets, or profiler integration.
- Consumer-specific numeric thresholds or hardware-specific tuning advice.
- Replacing the downstream rollout guide with a performance manual.

## Decisions

### 1. Start with docs and templates, not lints

**Choice:** Add a doc-first performance review kit instead of compile-time enforcement.

**Rationale:** Tiger Style performance rules are strongest during design and review: napkin math, batching, control-plane/data-plane separation, and cost × frequency ranking all depend on workload intent. A public guide and template can improve reviews immediately without pretending these rules are statically decidable.

**Alternative:** Add performance lints first. Rejected because the highest-value rules are not syntax-local and would produce noisy or shallow warnings.

### 2. Split explanatory guidance from copyable artifacts

**Choice:** Add `docs/performance-review.md` for guidance and `templates/performance-sketch.md` for a reusable fill-in artifact.

**Rationale:** Readers need two surfaces with different jobs. The guide explains how to think. The template gives maintainers a concrete structure they can copy into downstream design notes, review notes, or OpenSpec evidence.

**Alternative:** Put everything in `README.md` or inline examples in `openspec/specs/performance/spec.md`. Rejected because the README should stay a landing page and the spec is a contract, not the day-to-day review document.

### 3. Make the checklist map directly to Tiger Style performance rules

**Choice:** The guide keeps these exact second-level headings, each exactly once and in the listed order, with no extra second-level headings:
- `## Workload and hot path`
- `## Napkin math`
- `## Resource sketch: network, disk, memory, CPU`
- `## Cost × frequency ranking`
- `## Control plane vs data plane`
- `## Batching, limits, queues, and backpressure`
- `## CPU predictability and compiler luck`
- `## Measurement and validation`

The template keeps these exact second-level headings, each exactly once and in the listed order, with no extra second-level headings:
- `## Workload or entry point`
- `## Assumptions and units`
- `## Bandwidth and latency sketch`
- `## Resource budgets or limits`
- `## Cost × frequency ranking`
- `## Control-plane vs data-plane split`
- `## Batching, limits, queues, and backpressure`
- `## Measurement or validation plan`
- `## Open risks and non-goals`

Within those headings, every template section is a fill-in field or prompt rather than free-form narrative prose, including workload, assumptions, bandwidth/latency math, resource budgets or limits, cost × frequency, control/data-plane split, batching/backpressure, validation, and open risks. Under every guide heading, the body text includes at least one reviewer prompt or checklist question and explains why the checklist matters. The guide also says, in its overall guidance rather than per-heading boilerplate, that static lints or benchmarks do not replace workload reasoning.

Section-to-topic mapping stays explicit:
- `## Workload and hot path` owns early performance design before profiling.
- `## Resource sketch: network, disk, memory, CPU` owns network/disk/memory/CPU reasoning with both bandwidth and latency.
- `## Measurement and validation` owns the handoff from sketches to measurement or profiling.
- `## Cost × frequency ranking`, `## Control plane vs data plane`, and `## Batching, limits, queues, and backpressure` own the remaining design-time review obligations named in the performance spec.
- `## CPU predictability and compiler luck` explicitly covers large work chunks alongside predictability and reduced dependence on compiler optimization luck.

**Rationale:** The change should close the missing public-surface gap, not paraphrase performance advice loosely. Exact heading contracts make deterministic verification possible and keep the guide/template aligned with the performance spec.

**Alternative:** Publish a shorter generic checklist. Rejected because it would silently drop the specific Tiger Style rules this change is meant to expose.

### 4. Put discoverability in the README and contributor crosswalk

**Choice:** Update `README.md` to add these exact Markdown links:
- `[Performance review guide](docs/performance-review.md)`
- `[Performance sketch template](templates/performance-sketch.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links:
- `[Performance review guide](performance-review.md)`
- `[Performance sketch template](../templates/performance-sketch.md)`
- `[Performance spec contract](../openspec/specs/performance/spec.md)`

**Rationale:** The guide is only useful if readers can find it from the documented entry points they already use, and contributor docs must show where the public wording ultimately comes from.

**Alternative:** Link it only from the new guide or only from archived OpenSpec notes. Rejected because that keeps the performance material effectively hidden.

### 5. Specify deterministic proof and the human/oracle gate explicitly

**Choice:** Deterministic verification uses exact line checks:
- `docs/performance-review.md` must contain every exact heading from Decision 3 exactly once, in order, as standalone `## ...` lines with no extra second-level headings.
- `templates/performance-sketch.md` must contain every exact heading from Decision 3 exactly once, in order, as standalone `## ...` lines with no extra second-level headings.
- `README.md` must contain the exact Markdown-link lines `[Performance review guide](docs/performance-review.md)` and `[Performance sketch template](templates/performance-sketch.md)`, and deterministic proof must also show `docs/performance-review.md` plus `templates/performance-sketch.md` exist.
- `docs/contributor-guide.md` must contain the exact Markdown-link lines `[Performance review guide](performance-review.md)`, `[Performance sketch template](../templates/performance-sketch.md)`, and `[Performance spec contract](../openspec/specs/performance/spec.md)`, and deterministic proof must also show `docs/performance-review.md`, `templates/performance-sketch.md`, and `openspec/specs/performance/spec.md` exist.
- `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt` must exist after the human/oracle checkpoint and must contain these exact labeled lines:
  - `reviewer role: Tiger Style maintainer`
  - `reviewed artifact paths: docs/performance-review.md, templates/performance-sketch.md`
  - `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
  - `question 2: <pass|fail> - guide covers every required performance topic from Public Tiger Style Performance Review Guide under the designated headings`
  - `question 3: <pass|fail> - template covers every required field, including resource budgets or limits per relevant resource, and every section stays a fill-in field or prompt rather than free-form prose`
  - `question 4: <pass|fail> - overall guide guidance, not only per-section boilerplate, says static lints and benchmarks do not replace workload reasoning`
  - `verdict: <pass|fail>`

The human/oracle checkpoint is owned by a Tiger Style maintainer. The saved review record uses those exact lines and answer tokens. Deterministic proof must verify the exact lines plus the `pass` answers. For acceptance, questions 1 through 4 must all be `pass` and `verdict` must be `pass`; any failed question blocks completion until the docs are corrected.

**Rationale:** Grep can prove the skeleton exists. It cannot prove the guide avoids shallow cargo-cult phrasing or false promises about static enforcement.

**Alternative:** Use deterministic checks only. Rejected because this change is mostly about the quality and completeness of written guidance.

## Risks / Trade-offs

**[Docs without enforcement]** -> The guide may feel weaker than lints. Mitigation: document explicitly that this change is a first public review kit, not the final word on performance automation.

**[Checklist drift]** -> The guide could drift from the main performance spec over time. Mitigation: keep `docs/contributor-guide.md` pointing back to `openspec/specs/performance/spec.md` and verify the exact headings and link targets deterministically.

**[Scope creep into benchmarking]** -> Reviewers may expect benchmark tooling in the same change. Mitigation: keep benchmark and CI perf gates explicitly out of scope in proposal, design, and tasks.
