## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for effect capability boundaries.
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


### 3. Reference capability-oriented Rust crates before inventing examples

**Choice:** Cite `cap-std`, `cap-directories`, and `ambient-authority` as prior art for explicit authority values and non-ambient filesystem/network/clock access.

**Rationale:** These crates already encode the Rust ecosystem vocabulary for capability-based APIs. Tigerstyle can reference their patterns without making every consumer adopt them.

**Implementation:** Standards examples should show local Tigerstyle capability traits and effect enums first, then note when a shell crate MAY use `cap-std`/`cap-directories` handles instead of raw `std::fs`/`std::net`. Use `ambient-authority` as a named example of an intentionally loud escape hatch.

**Alternative:** Require `cap-std` in consumers. Rejected because Tigerstyle is a lint/standards tool and must support projects that already have their own shell abstractions.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
