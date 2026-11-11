## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for content addressed evidence.
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


### 3. Reuse existing evidence/hash ecosystem crates

**Choice:** Use the existing `blake3` dependency for digest computation and reference `cargo_metadata` for workspace/package identity when implementation needs stable Cargo graph context. Keep JSON evidence on `serde`/`serde_json`, which the runner already uses.

**Rationale:** Tigerstyle already depends on `blake3` in `cargo-tigerstyle`; adding a second hashing stack would fragment evidence IDs. `cargo_metadata` is the standard way to identify packages, targets, and workspace roots without scraping `cargo` text output.

**Implementation:** Evidence schemas should name digest algorithm/version fields explicitly (`blake3`, normalization version, config/tool hash inputs). If SARIF output needs richer modeling later, evaluate a SARIF crate only after proving the existing hand-written output cannot carry the required fingerprint fields.

**Alternative:** Shell out to `cargo metadata` and hash raw source bytes only. Rejected because token/path/config normalization needs structured inputs and stable schema fields.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
