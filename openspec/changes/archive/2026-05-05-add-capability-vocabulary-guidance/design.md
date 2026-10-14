## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for capability vocabulary guidance.
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


### 3. Align vocabulary with existing capability and secret-handling crates

**Choice:** Reference `cap-std`, `cap-directories`, and `ambient-authority` for filesystem/network/clock authority vocabulary; reference `secrecy`/`zeroize` as examples for credential-bearing capability values.

**Rationale:** Capability names such as `Clock`, `Entropy`, `ObjectStore`, `CredentialReader`, and `RemoteExecutor` should feel Rust-native and auditable. Existing crates provide useful naming and safety precedent even when Tigerstyle consumers implement local traits.

**Implementation:** Guidance should show local traits/tokens first, then optional adapters over ecosystem crates. Credential examples MUST avoid exposing `Display`/`Debug` secrets and SHOULD document zeroization/redaction expectations.

**Alternative:** Provide a Tigerstyle runtime capability crate immediately. Rejected for this change because the first deliverable is standards vocabulary and lintable boundaries, not a runtime framework.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
