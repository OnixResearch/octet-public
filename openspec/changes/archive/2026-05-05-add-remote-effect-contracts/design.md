## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for remote effect contracts.
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


### 3. Reference service, retry, and test-double crates without coupling the standard

**Choice:** Reference `tower::Service`/layers as prior art for bounded service middleware, and reference `wiremock` or `httpmock` for deterministic remote-handler tests. Consider `uuid`/`ulid`-style crates for idempotency keys only in consumer examples that already need globally unique request IDs.

**Rationale:** Remote effects need explicit timeouts, retry budgets, idempotency keys, receipts, and replay semantics. The `tower` ecosystem models middleware composition well, but Tigerstyle should not require a specific async stack.

**Implementation:** The standard should define required fields and verification behavior. Implementations MAY map those fields onto `tower` layers or existing client middleware. Tests SHOULD use local deterministic fakes or HTTP test doubles rather than live network services.

**Alternative:** Standardize on `reqwest` or one HTTP client. Rejected because remote effects also include queues, subprocesses, RPC, and forge APIs.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
