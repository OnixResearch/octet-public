## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for deterministic handler testing.
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


### 3. Use established deterministic testing crates where they fit

**Choice:** Reference `proptest` for property coverage, `insta`/`expect-test` for stable snapshots, `tokio-test` for async shell tests, and `wiremock`/`httpmock` for local remote-service doubles.

**Rationale:** Handler testing should prove behavior across valid and invalid paths without live clocks, random sources, stores, or networks. Existing crates cover input generation, snapshot review, async execution, and HTTP doubles.

**Implementation:** Pure effect handlers should still use simple in-memory fakes by default. Add external test crates only when they reduce evidence noise or cover a class of behavior that table tests cannot. Snapshot artifacts MUST stay deterministic and reviewable.

**Alternative:** Mock everything through a generic mocking framework. Rejected as the default because explicit fake handlers better expose Tigerstyle capability boundaries and invalid-path cleanup obligations.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
