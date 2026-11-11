## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for top level purity and delayed effects.
- Keep enforcement staged until deterministic fixtures prove checker quality.
- Make OpenSpec evidence and policy classifications explicit before implementation.

**Non-Goals:**
- Recreate Unison's language, codebase manager, or runtime.
- Add broad default hard gates without rollout evidence.
- Depend on networked services for verification.

## Decisions

### 1. Start with docs and policy classification

**Choice:** Implement guidance and policy metadata before new hard lints.

**Rationale:** The Unison concept is valuable as a design discipline first; checkers need fixtures and downstream inventory before hard gating.

**Alternative:** Add a lint immediately. Rejected because the static signal must be proven per scenario.

### 2. Require deterministic evidence for promotion

**Choice:** Every future checker or command behavior gets positive, negative, and output-contract evidence before completion.

**Rationale:** Tigerstyle standards should not drift into prose-only claims.


### 3. Reference capability crates for shell escape hatches, not core purity

**Choice:** Cite `cap-std`/`ambient-authority` as examples of explicit authority at shell boundaries, while keeping pure-core enforcement based on Rust modules/crates, lint analysis, and dependency classification.

**Rationale:** Top-level purity is primarily a standards/lint concern: constructors, globals, and lazy statics must not hide IO/time/env/network access. Capability crates are useful examples for shell code but cannot by themselves prove Tigerstyle core purity.

**Implementation:** Examples should prefer injected values and delayed effect descriptors. If a shell example needs filesystem access, prefer a directory handle/capability example over an ambient path example.

**Alternative:** Recommend `once_cell`/`lazy_static` wrappers for delayed effects. Rejected as a default because those crates can make hidden top-level effects easier; Tigerstyle should instead require explicit task/command/effect construction.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
