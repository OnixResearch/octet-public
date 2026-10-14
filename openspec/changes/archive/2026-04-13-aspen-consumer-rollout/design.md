# Design: Integrate Tigerstyle Into Aspen

## Context

`tigerstyle` already compiles, registers the lint catalog, and has baseline
triage results for `aspen`, `verified-logic`, and `clankers`. What it does not
have is a downstream adoption path proven in a real workspace.

The open `custom-clippy-lints` change still lists two downstream tasks:
adding tigerstyle as a flake check in aspen and configuring per-project
allowlists. The baseline triage shows why this needs its own rollout plan.
Aspen currently has hundreds of hits in several families: 299 ignored results,
421 unwrap/expect calls, 323 panic-family calls, 510 unchecked narrowings,
and 173 unbounded loops. A naive full-catalog enablement would generate more
heat than signal.

## Goals / Non-Goals

**Goals:**

- Prove one real consumer integration end to end, starting with aspen
- Define staged rollout that starts with actionable, lower-noise lints
- Keep workspace-specific tuning in the consumer repo rather than hardcoding
  aspen policy into tigerstyle defaults
- Capture validation evidence future consumers can reuse

**Non-Goals:**

- Enabling every tigerstyle lint in aspen on day one
- Rolling out clankers and verified-logic in the same change
- Changing lint semantics unless rollout uncovers a concrete bug worth a
  separate change

## Decisions

### 1. Use aspen as the proving ground

**Choice:** First rollout targets aspen only.

**Rationale:** Aspen is the primary consumer named in project context and
already has triage evidence. One real integration is more useful than a
generic downstream guide with no proof.

**Alternative:** Write generic downstream docs first. Rejected because it
would not flush out flake, config, or tooling problems.

**Implementation:** Keep this change scoped to aspen, then reuse the pattern
for later consumers.

### 2. Start with a starter lint set, not the whole catalog

**Choice:** Initial aspen integration enables only a small set of low-noise,
high-signal lints and explicitly defers noisy families.

**Rationale:** Aspen has hundreds of findings in several categories. A narrow
starter set gives immediate value without forcing broad cleanup or blanket
suppression.

**Alternative:** Gate the whole catalog immediately. Rejected because the
existing counts show that would bury signal.

**Implementation:** Pick starter lints from baseline evidence, record deferred
families, and define promotion criteria for later phases.

### 3. Keep tuning in aspen's `dylint.toml`

**Choice:** Thresholds and allowlists live in aspen, not in tigerstyle.

**Rationale:** Function-length limits, acronym exceptions, and justified
allowlists are workspace policy. Baking aspen-specific choices into tigerstyle
would distort other consumers.

**Alternative:** Add more tigerstyle defaults or hardcoded repo exceptions.
Rejected because it couples the lint crate to one consumer.

**Implementation:** Tigerstyle documents expected knobs; aspen owns actual
thresholds and exception comments.

### 4. Treat validation evidence as required work product

**Choice:** Rollout is not done until we capture an end-to-end aspen run with
chosen config and starter lint set.

**Rationale:** Downstream readiness claims need evidence, not only planned
commands.

**Alternative:** Consider the work done once config files exist. Rejected
because it misses toolchain and flake integration failures.

**Implementation:** Save command lines, result summary, representative
findings, and deferred lint families in the change artifacts or linked notes.

## Risks / Trade-offs

**[Starter set too weak]** Initial rollout may look timid.
**Mitigation:** Record why each deferred lint is out of the first phase and
what evidence would justify promotion.

**[Hidden tooling friction]** Aspen's nix or dylint environment may still fail
before lint output appears.
**Mitigation:** Validate the full command path early and capture environment
requirements in rollout notes.

**[False-positive drift]** Some supposedly low-noise lints may still need
fixes.
**Mitigation:** Split lint bugs into separate follow-up changes instead of
papering over them with blanket allows.
