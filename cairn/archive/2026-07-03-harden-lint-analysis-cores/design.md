## Context

Snippet-based checks are sometimes unavoidable for macro callsites and diagnostics, but they should not be the primary semantic classifier when HIR or type facts exist. The target lints currently mix traversal, source extraction, classification, and emission in ways that make negative cases harder to test.

## Decisions

### 1. Split every target lint into facts, pure classifier, and emission

**Choice:** Each refactored lint defines a small fact DTO, a pure `classify_*` function, and a thin rustc visitor that gathers facts and emits diagnostics.

**Rationale:** This follows the repo's functional-core / imperative-shell rule and makes false-positive fixes testable with ordinary assertions.

### 2. Prefer HIR/type facts over string matching

**Choice:** Use HIR item kinds, safety markers, visibility, ADT paths, loop sources, lock-field type resolution, and literal context before falling back to snippets.

**Rationale:** Structured facts are less brittle under formatting, aliases, and macro expansion.

### 3. Allow documented snippet fallbacks only at boundaries

**Choice:** Snippet fallbacks remain allowed for macro callsite recovery, insertion spans, and diagnostic text, but each fallback must have a named reason and guardrail fixture.

**Rationale:** Some rustc APIs do not preserve the original spelling needed for suggestions. The fallback should be contained and visible.

## Target lints

- `public_unsafe_api`: replace `unsafe fn` / `extern` substring checks with HIR safety and ABI facts.
- `multi_lock_ordering`: resolve lock field types where possible and distinguish comments that actually document lock order.
- `unbounded_loop`: recognize structural counters, breaks, shutdown receivers, and bounded iterator forms before source-marker fallback.
- `magic_number`: classify policy thresholds using expression context and names from bindings/config rather than a tiny domain-word list.

## Validation

Each target lint gets pure positive and negative classifier tests plus UI fixtures proving unchanged current intended diagnostics and newly clean guardrail cases.
