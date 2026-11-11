## Context

`no_panic` is now a true tool lint, so `#[allow(tigerstyle::no_panic)]`
suppresses the panic diagnostic before custom justification logic can run. The
repo already handles that by routing escape-hatch review through the companion
lint `unjustified_no_panic_allow`.

Current `src/safety/no_panic.rs` only emits that companion lint while walking an
expanded panic-family macro. For this change, that closed macro set stays:
`todo!()`, `panic!()`, `unimplemented!()`, and `unreachable!()`.

That preserves the live escape-hatch contract for justified `todo!()` stubs,
but it misses the cleanup case where the panic macro is gone and the allow attr
stays behind.

This change is a hygiene fix, not a redesign of `no_panic` semantics. It needs
to add orphaned-allow detection without regressing the existing justified
`todo!()` path or broadening the escape hatch beyond the documented
`tigerstyle::no_panic` form.

## Goals / Non-Goals

**Goals:**
- Detect orphaned direct `#[allow(tigerstyle::no_panic)]` attrs on supported
  executable item bodies and block expressions
- Keep justified live `todo!()` escapes silent exactly as they are today
- Keep all escape-hatch diagnostics on `unjustified_no_panic_allow`
- Keep help/note rendering on the shared `src/diagnostics.rs` path
- Prove both orphaned item-level and orphaned block-level cases with
  deterministic UI fixtures

**Non-Goals:**
- Auditing bare compatibility attrs like `#[allow(no_panic)]`
- Auditing container-level attrs on `mod`, `impl`, trait, or crate nodes
  without a direct executable body check
- Diagnosing nested but still-live outer allows as “too broad”
- Parsing inactive `cfg_attr(...)` branches from source text

## Decisions

### 1. Scope stays on direct tool-scoped attrs for executable bodies

**Choice:** Audit only realized `#[allow(tigerstyle::no_panic)]` attrs that are
attached directly to:
- items with executable bodies,
- impl items with bodies,
- trait items with default bodies,
- block expressions.

A `cfg_attr(...)` is in scope only when rustc expands it into a direct
`#[allow(tigerstyle::no_panic)]` on one of those supported nodes.

Bare `#[allow(no_panic)]`, crate-level attrs, and container attrs on `mod`,
`impl`, or trait declarations stay out of scope for this change.

**Rationale:** The documented escape hatch already uses the canonical tool name.
Limiting the hygiene pass to direct executable bodies keeps the implementation
small and avoids mixed-phase ambiguity around compatibility aliases and
container-wide lint levels.

**Implementation:** Reuse the existing attr matcher for the canonical
`tigerstyle::no_panic` form and walk supported HIR nodes directly instead of
adding broader source-text scans.

**Alternative:** Audit every rustc-recognized `allow(no_panic)` spelling and
container-level attr shape. Rejected because it turns a hygiene fix into a much
larger source-attr compatibility project.

### 2. Orphan detection is a second pass over candidate allow attrs

**Choice:** Treat an allow attr as live when its annotated subtree still
contains at least one panic-family macro call. Treat it as orphaned when that
subtree contains none.

Nested inner allows still count as “live” for an outer allow if the outer
annotated subtree still contains a panic-family macro anywhere inside it. This
change does not try to prove that the outer allow is the narrowest possible
scope.

**Rationale:** The change is about dead escape hatches left behind after cleanup,
not about minimizing the scope of still-active escapes. Counting any remaining
panic-family macro in the annotated subtree avoids false orphan reports on
nested allow structures.

**Implementation:**
- collect supported `#[allow(tigerstyle::no_panic)]` attr spans as candidate
  escape hatches
- keep the current macro-driven path for live misuse checks
- record which candidate attr spans still cover a panic-family macro
- after traversal, emit `unjustified_no_panic_allow` once for each candidate
  attr span that was never marked live

**Alternative:** Treat only the nearest enclosing allow as live and warn on any
outer allow that no longer has a unique macro under it. Rejected because that
would misclassify still-effective outer attrs as orphaned and requires more
shadowing analysis than this fix needs.

### 3. Existing live-path behavior stays stable; orphaned attrs get one new message

**Choice:** Keep the current messages unchanged for the existing live cases:
- missing adjacent `//` justification comment →
  `` `#[allow(tigerstyle::no_panic)]` needs an adjacent `//` justification comment and may only wrap temporary `todo!()` stubs ``
- wrong macro under the allow →
  `` `#[allow(tigerstyle::no_panic)]` is reserved for temporary `todo!()` stubs, not `<macro>` ``

Add one new orphan-specific primary message:

```
`#[allow(tigerstyle::no_panic)]` no longer covers any panic-family macro; remove it
```

The help and note text stay exactly on the existing
`unjustified_no_panic_allow` metadata entry in `src/diagnostics.rs`:

- help: `Tiger Style (Documented Panic Escape): reserve #[allow(tigerstyle::no_panic)] for temporary todo!() stubs with an adjacent // justification comment Example: // justified stub while API lands\n#[allow(tigerstyle::no_panic)] fn stub() { todo!() }`
- note: `see Tiger Style guide: Error Handling / Panics`

**Rationale:** Existing snapshots for live justified/unjustified cases should
not churn. Orphaned attrs are a distinct state, and reusing the missing-comment
message would be wrong when the attr still has a valid adjacent comment.

**Implementation:** Emit the orphan diagnostic at the attr span through the same
`emit_tiger_style_lint(..., UNJUSTIFIED_NO_PANIC_ALLOW, ...)` path used by the
current companion lint.

**Alternative:** Reuse the current missing-comment wording for orphaned attrs.
Rejected because a commented orphan is not missing a justification comment; it
is stale and should be removed.

### 4. Preservation behavior is explicit

**Choice:** A justified live `todo!()` under a supported
`#[allow(tigerstyle::no_panic)]` stays completely silent: neither `no_panic`
nor `unjustified_no_panic_allow` emits.

The existing misuse cases also stay unchanged:
- `todo!()` without the adjacent `//` comment still warns
- `panic!()`, `unimplemented!()`, and `unreachable!()` under the allow still
  warn even when a comment is present

**Rationale:** The whole point of this change is to catch stale escape hatches
without reopening the already-settled temporary-stub contract.

**Implementation:** Keep the current nearest-enclosing-allow logic for live
macro checks. Orphan detection runs only after that live-path accounting is
complete.

## Verification strategy

- Keep `ui_tests/no_panic_todo_allow.rs(.stderr)` as the preservation fixture:
  justified function-level and block-level `todo!()` escapes stay quiet, while
  unjustified `todo!()` and non-`todo!()` macro uses still warn with the current
  wording.
- Add `ui_tests/no_panic_orphan_allow.rs(.stderr)` with both orphaned item-level
  and orphaned block-level attrs.
- Make the orphan fixture cover both commented and uncommented orphan attrs so
  the new orphan message is proven to be comment-independent.
- Include one `cfg_attr(...)`-expanded orphan case in that fixture so the
  post-expansion scope rule is exercised, not just documented.
- Run `cargo test --lib`.
- Run `cargo test --test ui -- --ignored --nocapture`.

## Risks / Trade-offs

**[Container attr gap]** → A repo could still leave a stale container-level
`#[allow(tigerstyle::no_panic)]` outside this audit scope. Mitigation: keep that
shape out of scope here and open a separate change only if a real consumer needs
it.

**[Nested allow ambiguity]** → An outer allow can stay live even if an inner
allow would be enough. Mitigation: accept that trade-off; this change only
removes dead attrs, not broad-but-live ones.

**[Macro-accounting drift]** → Orphan detection could disagree with the live
misuse path if the panic-family macro set diverges. Mitigation: reuse the same
panic-macro list and attr-span accounting in one module.
