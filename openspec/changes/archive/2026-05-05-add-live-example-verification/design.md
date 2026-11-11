## Context

This change imports one focused Unison-inspired concept into Tigerstyle's Rust-first standards. The implementation should preserve Tigerstyle's FCIS model: pure core, explicit data, bounded effects, deterministic evidence, and local-first verification.

## Goals / Non-Goals

**Goals:**
- Add reviewable standards language for live example verification.
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


### 3. Build on Rust's executable documentation tools

**Choice:** Treat `rustdoc` doctests and `cargo test --doc` as the default live-doc mechanism; reference `trybuild` for compile-fail examples and `mdbook` only for book-style documentation integration.

**Rationale:** Rust already has executable examples in documentation. Tigerstyle should standardize how examples are discovered, bounded, and reported rather than inventing a parallel runner.

**Implementation:** Positive examples SHOULD be ordinary doctests or fixture crates. Compile-fail policy examples MAY use `trybuild`-style fixtures or the existing repo UI harness, whichever keeps snapshots more stable. Keep `assert_cmd`/`snapbox`-style CLI transcript checks in the runner test layer where they already fit.

**Alternative:** Parse every Markdown fence with a custom runner first. Rejected because rustdoc already defines Rust code-fence semantics and avoids many edition/import edge cases.

## Risks / Trade-offs

**Concept overreach** → Keep this change scoped to the listed requirements and classify weak signals as advisory.

**False confidence** → Documentation must distinguish design guidance from proof.

**Consumer friction** → Default checks must not require new heavyweight tools unless a profile or command is explicitly selected.

## Validation Plan

- Validate OpenSpec strict parsing for this change.
- Add deterministic docs/policy checks for every scenario.
- Add future implementation fixtures before any hard-gate promotion.
