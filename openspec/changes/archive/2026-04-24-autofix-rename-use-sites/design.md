## Context

Wave-1 autofix added structured suggestions to 24 lints. Rename lints
(`bool_naming`, `acronym_style`, `negated_predicate`) use `MaybeIncorrect` or
`HasPlaceholders` because they only rewrite the declaration span. Applying the
suggestion breaks compilation until every use site is manually renamed.

For local variable bindings, all use sites live within the enclosing function
body and are visible through the HIR. This is the narrowest possible rename
scope and the safest candidate for `MachineApplicable` promotion.

## Goals / Non-Goals

**Goals:**
- emit multipart suggestions that rewrite declaration + all use sites for local
  variable patterns in `bool_naming` and `negated_predicate`
- promote those patterns to `MachineApplicable` so `cargo fix` applies them
- prove correctness through `.fixed` fixtures and the autofix test rail
- share use-site collection logic between the two lints

**Non-Goals:**
- item-level or function-level use-site collection (crate-wide scope, public API
  breakage risk)
- `acronym_style` promotion (too many item kinds, separate change)
- cross-crate reference rewriting (impossible from a Dylint lint)

## Decisions

### 1. Use-site collection via HIR body visitor

**Choice:** walk the enclosing `Body` HIR with a visitor that collects every
`hir::Path` segment resolving to the flagged binding's `HirId`.

**Rationale:** local variable bindings are resolved via `HirId`, not `DefId`.
The `typeck_results().qpath_res()` or `node_type()` APIs can confirm resolution.
Walking the enclosing body (not the entire crate) is O(function size) and
matches the natural scope of a let binding.

**Alternative rejected:** `tcx` global name resolution. Overkill for locals
because locals don't have `DefId`s in the global resolver; they're HIR-only.

### 2. Multipart suggestion via `multipart_suggestion`

**Choice:** emit a `MultipartSuggestion` with one substitution per span
(declaration + each use site) rather than N separate single suggestions.

**Rationale:** `cargo fix` applies all parts atomically. Multiple single
suggestions for the same lint firing would conflict and produce partial renames.

**Implementation:** the `LintSuggestion` enum in `src/diagnostics.rs` already
supports `Multi` variant. Each collected span gets a substitution entry with the
renamed identifier.

### 3. Macro-expansion fallback

**Choice:** if the declaration span or any use-site span has
`span.from_expansion() == true`, fall back to `MaybeIncorrect` and emit a
declaration-only single suggestion (current behavior).

**Rationale:** macro-generated spans may not be rewritable by `cargo fix`, and
the emitted code may not reflect what the user wrote. Conservative fallback
preserves the wave-1 contract.

### 4. Shared collector as a function, not a trait

**Choice:** a standalone `fn collect_local_use_sites(cx, body, hir_id) -> Vec<Span>`
in a shared module, called by both `bool_naming` and `negated_predicate`.

**Rationale:** both lints need the same walk. A free function is simpler than a
trait or visitor struct that both lints must instantiate. If a third lint needs
this later, the function signature stays stable.

## Risks / Trade-offs

**Closure captures:** a let binding captured by a closure inside the function
body still resolves to the same `HirId`. The visitor will collect these use
sites. The multipart suggestion will rename them. This is correct behavior --
closures within the same function body are in scope.

**Shadowing:** if a nested scope shadows the binding name, the HIR resolver
distinguishes them by `HirId`. The collector will only find references to the
original binding, not the shadow. This is correct.

**Pattern bindings in match arms:** `bool_naming` fires on `check_local` (let
bindings), not match arm patterns. Use-site collection starts from the let
binding's `HirId` and won't accidentally collect match-arm bindings with the
same name.

**Performance:** walking one function body per flagged binding is negligible.
Even a pathological case with hundreds of flagged bindings in one function adds
sub-millisecond overhead.
