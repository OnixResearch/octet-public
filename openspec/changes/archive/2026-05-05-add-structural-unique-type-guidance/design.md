## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for structural and unique type guidance.
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


### 3. Reference newtype and secret crates as implementation aids

**Choice:** Cite `nutype`, `derive_more`, typed-ID crates, `uuid`, `secrecy`, and `zeroize` as implementation references for nominal IDs, units, secrets, protocol states, and capability tokens.

**Rationale:** Tigerstyle should require the pattern, not one macro crate. Projects differ on proc-macro budgets, compile-time tolerance, and public API stability.

**Implementation:** Examples should start with plain tuple structs and hand-written constructors, then mention derive/newtype crates as optional boilerplate reducers when they preserve invariants. Secret wrappers should avoid accidental `Debug`/`Clone`/serialization and document erasure/redaction behavior.

**Alternative:** Require a specific newtype macro. Rejected because macro-generated trait impls can weaken invariants if chosen carelessly.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
