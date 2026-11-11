## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for semantic refactor evidence.
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


### 3. Reference rust-analyzer crates for semantic identity spikes

**Choice:** Reference `ra_ap_syntax`, `ra_ap_ide`, and related rust-analyzer crates as optional spike dependencies for symbol identity and rename/move evidence; use `cargo_metadata` for workspace graph/package identity.

**Rationale:** Semantic refactor evidence needs more than text diffs, but a full rust-analyzer dependency is large and toolchain-sensitive. It should be evaluated behind an adapter, not embedded into the base lint path prematurely.

**Implementation:** Initial implementation MAY use compiler/HIR information already available in Dylint for lint-local identities and `cargo_metadata` for package context. A later spike can compare rust-analyzer-derived symbol IDs against Tigerstyle fingerprints and old/new mappings.

**Alternative:** Depend directly on rust-analyzer internals in the default runner. Rejected because version churn and compile cost are likely high for normal `cargo tigerstyle check` users.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
