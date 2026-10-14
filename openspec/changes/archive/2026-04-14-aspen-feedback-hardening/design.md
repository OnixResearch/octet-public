# Design: Harden Tigerstyle From Aspen Rollout Findings

## Context

Aspen gave `tigerstyle` its first real downstream proving ground. That rollout
already produced durable artifacts: phased starter lints, workspace-local
`dylint.toml` tuning, validation evidence, and follow-up changes for noisy
packages. It also exposed three product-level gaps:

1. downstream setup still depends on Aspen-specific wrapper knowledge for
   nix-managed nightly + Dylint
2. the current rollout story does not yet package starter profiles, config
   templates, and promotion checklists for reuse
3. `ambient_clock` and the high-noise safety families still need clearer
   classification guidance before later consumers can adopt them efficiently

This change turns those Aspen findings into reusable `tigerstyle` guidance.

## Goals / Non-Goals

**Goals:**

- Generalize Aspen rollout lessons into a reusable downstream integration kit
- Keep rollout staged with named starter profiles and explicit promotion rules
- Clarify `ambient_clock` boundary ownership patterns that appeared in Aspen
- Define an inventory-first workflow for high-noise lint families

**Non-Goals:**

- Re-tuning every noisy lint family in this change
- Hardcoding Aspen's thresholds or allowlists into `tigerstyle` defaults
- Replacing consumer-owned wrapper scripts with automatic toolchain detection

## Decisions

### 1. Generalize Aspen artifacts, do not hardcode Aspen policy

**Choice:** Reuse Aspen's findings as examples and templates, but keep consumer
thresholds and allowlists in the consumer repo.

**Rationale:** Aspen needed `function_length_max = 100`,
`nesting_max_depth = 4`, `assertion_density_min_lines = 30`, and
`acronym_allowlist = ["IO"]`. Those are real rollout lessons, but they are not
universal defaults.

**Alternative:** Bake Aspen values into `tigerstyle` defaults. Rejected because
it would distort other consumers and hide workspace policy inside the lint
crate.

**Implementation:** Ship commented templates and review checklists, not
hardcoded repo exceptions.

### 2. Name starter profiles explicitly

**Choice:** Document named rollout profiles instead of saying "start small."

**Rationale:** Aspen's real phase-1 path (`ambient_clock`,
`compound_assertion`, `contradictory_time`) was more useful than the earlier
raw triage ranking because it optimized for signal and rollout friction, not
just severity.

**Alternative:** Leave profile selection as free-form guidance. Rejected
because later consumers would have to rediscover why some low-noise lints make
better first adopters than some higher-severity but noisier families.

**Implementation:** Document starter profiles, expected package scope, and
promotion gates.

### 3. Treat toolchain wrappers as reusable recipes

**Choice:** Capture the Aspen wrapper pattern as a supported downstream recipe,
not as an Aspen-only note.

**Rationale:** The first failure mode future users will hit is often setup, not
lint semantics. Aspen showed that nix-managed toolchains can require wrapped
`rustup`, `cargo`, and `dylint-link`, plus temporary `CARGO_HOME` patching.

**Alternative:** Wait for upstream Dylint packaging/toolchain fixes and avoid
writing guidance now. Rejected because later consumers need a working path
before upstream cleanup lands.

**Implementation:** Provide a reusable check-path template and document the
minimum environment assumptions.

### 4. Expand `ambient_clock` guidance around boundary patterns

**Choice:** Add boundary-pattern guidance for timeout/deadline conversion,
elapsed-time measurement, and shell-owned wait loops.

**Rationale:** Aspen's coordination crate concentrated `ambient_clock` warnings
in exactly those shapes. Maintainers need to know when to inject time, when to
centralize one boundary helper, and when a narrow item-scoped allow is the
right move.

**Alternative:** Leave the current generic lint message as-is. Rejected because
it does not give enough guidance for repeated timeout/retry patterns.

**Implementation:** Add examples and decision rules tied to the existing
`ambient_clock` and `contradictory_time` story.

### 5. Require inventory before promoting noisy families

**Choice:** Promotion guidance for `ignored_result`, `no_unwrap`, `no_panic`,
`unchecked_narrowing`, and `unbounded_loop` starts with per-crate inventories.

**Rationale:** Aspen's counts were too large for blanket enablement. Inventory
first keeps later promotion tied to real owners, real categories, and bounded
warning sets.

**Alternative:** Recommend enabling the family and cleaning up reactively.
Rejected because that encourages blanket `allow`s and hides false-positive
patterns.

**Implementation:** Document required evidence, known noise patterns, and
promotion checklists.

## Risks / Trade-offs

**[Aspen bias]** The kit may overfit Aspen.
**Mitigation:** Present Aspen as the first evidence-backed example while
keeping thresholds and scope configurable.

**[Wrapper drift]** Downstream toolchain workarounds may age badly.
**Mitigation:** Frame them as recipes with assumptions and update points, not
as opaque magic.

**[Too much process]** A heavy rollout playbook could slow adoption.
**Mitigation:** Keep the starter path short and reserve inventory work for the
high-noise families only.
