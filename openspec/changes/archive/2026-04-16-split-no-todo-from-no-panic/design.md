## Context

`no_panic` currently owns two different concerns:
- production panics that should never ship (`panic!()`, `unimplemented!()`,
  `unreachable!()`, and today also `todo!()`)
- a special-case documented stub escape hatch for `todo!()` that tigerstyle has
  to recover through `unjustified_no_panic_allow`

That coupling became the repo's clearest remaining tool-lint migration blocker.
As soon as `#[allow(tigerstyle::no_panic)]` suppresses the real tool lint,
custom logic can no longer enforce the intended `todo!()`-only contract without
running a second audit path. The repo already documented that this is a hard
semantic mismatch for bulk migration.

A second real-world constraint showed up in current Aspen state: live consumer
code still contains bare `#![allow(no_panic)]` crate attrs, while tigerstyle's
canonical docs talk about tool-scoped `#[allow(tigerstyle::no_panic)]`. The
follow-on design therefore needs both an honest lint split and an explicit
migration note for already-shipped bare spellings.

## Goals / Non-Goals

**Goals:**
- give `todo!()` a dedicated tool lint whose name matches its contract
- keep `no_panic` focused on panic-family aborts that are not temporary stubs
- move the documented temporary-stub escape hatch onto `no_todo`
- keep escape-hatch auditing reviewable through one companion lint with exact,
  deterministic wording
- preserve safety-domain group membership and bare config compatibility for the
  new lint names
- document the downstream rename path for both canonical and bare allow attrs

**Non-Goals:**
- preserving old `allow(no_panic)` source attrs as silently working aliases for
  justified `todo!()` stubs
- introducing a migration-only lint just to point from `allow(no_panic)` to
  `allow(no_todo)`
- broadening the documented escape hatch beyond direct executable-body attrs
- changing starter-profile rollout classification beyond the `no_todo` split
- adding autofix support for the rename in this change

## Decisions

### 1. `todo!()` gets a dedicated safety lint; `no_panic` narrows to real panic-family aborts

**Choice:** Add a new true tool lint, `tigerstyle::NO_TODO`, and remove
`todo!()` from `no_panic` detection. After the split:
- `no_todo` owns `todo!()`
- `no_panic` owns `panic!()`, `unimplemented!()`, and `unreachable!()`

Both lints stay in the `tigerstyle::safety` domain group and continue using the
shared test-context suppression helper.

**Rationale:** The lint name should match the reviewed behavior. A dedicated
`no_todo` lint makes source-level attrs, diagnostics, config keys, and docs all
say the same thing.

**Implementation:** Add `src/safety/no_todo.rs`, register the new lint through
inventory + domain-group wiring, and trim `src/safety/no_panic.rs` to the three
remaining abort macros. Shared macro-callsite helpers may stay local or move
into a tiny internal helper if that reduces duplication.

**Alternative:** Keep `todo!()` inside `no_panic` and continue relying on the
companion audit lint. Rejected because it leaves the core semantic mismatch in
place and keeps downstream docs anchored to the wrong lint name.

### 2. The documented stub escape hatch moves to `no_todo`, with bare and tool-scoped source attrs supported

**Choice:** The documented temporary-stub attrs become
`#[allow(tigerstyle::no_todo)]` and the supported bare compatibility spelling
`#[allow(no_todo)]`, both with the existing adjacent `//` justification rule.
The companion audit lint becomes `tigerstyle::UNJUSTIFIED_NO_TODO_ALLOW`.

The audit scope stays on direct executable-body attrs that tigerstyle already
knows how to inspect: function items, impl items with bodies, trait default
items, and block expressions. `cfg_attr(...)` counts only when rustc expands it
into a direct supported attr on one of those nodes.

**Rationale:** Aspen already proves that bare source attrs exist in live
consumer code. Supporting both spellings for the new lint keeps the migration
practical while still giving the canonical tool-scoped form a truthful name.
Renaming the companion lint to match that attr keeps diagnostics, docs, and
config surfaces coherent.

**Implementation:** Reuse the current adjacent-comment/orphaned-attr machinery,
but retarget it to `no_todo`, teach it to recognize both supported source-attr
spellings, emit the new companion lint ID, and rewrite the matching metadata in
`src/diagnostics.rs`. For `cfg_attr(...)`, rely on the post-expansion late-pass
attr/HIR path already available under Dylint instead of introducing a new
pre-expansion contract.

**Alternative:** Keep the companion lint name
`unjustified_no_panic_allow` while moving the attr to `no_todo`. Rejected
because the audit lint name would then describe the wrong attr forever.

### 3. Legacy `allow(no_panic)` spellings become migration work, not compatibility semantics

**Choice:** After the split, `allow(no_panic)` /
`allow(tigerstyle::no_panic)` no longer silence `todo!()`. `no_todo` emits on
those stubs until the attr is renamed.

This change does not add a special migration-only warning when an old
`no_panic` allow still sits next to `todo!()`. Those old attrs become dead
migration work: they no longer justify the stub, and the new companion lint
only audits `no_todo` attrs.

**Rationale:** Silent grandfathering would reintroduce the same semantic blur
under a second name. A docs-driven rename is simpler and keeps lint behavior
truthful.

**Implementation:** `no_todo` only checks for `no_todo` attrs when deciding
whether a stub is justified. `no_panic` no longer treats `todo!()` specially,
and `unjustified_no_todo_allow` ignores legacy `no_panic` attrs. The rollout
docs and template comments carry the rename guidance for both bare and
tool-scoped forms.

**Alternative:** Make `no_todo` temporarily honor `allow(no_panic)` as a second
escape hatch. Rejected because it extends the awkward transition state and
encodes the old naming mismatch into new behavior.

### 4. Config, inventory, and group wiring stay explicit

**Choice:** The canonical lint inventory gains `no_todo` and
`unjustified_no_todo_allow`. `unjustified_no_panic_allow` leaves the active
inventory. Bare config keys remain supported for active canonical lint names,
so downstream `dylint.toml` can set `no_todo = "deny"` or disable
`unjustified_no_todo_allow` without spelling `tigerstyle::...`.

Both `no_todo` and `unjustified_no_todo_allow` stay warn-by-default and belong
to the `tigerstyle::safety` group alongside the narrowed `no_panic`.

**Rationale:** This repo's supported consumer contract keeps bare config keys
and a narrow set of bare source attrs through compatibility aliases. The split
should fit that pattern instead of inventing a special exception.

**Implementation:** Update `LINT_INVENTORY`, compatibility alias registration,
`src/diagnostics.rs`, diagnostic metadata snapshots, and any inventory
tests/fixtures that assert the published lint set.

**Alternative:** Keep deprecated aliases for `unjustified_no_panic_allow` in
active inventory. Rejected for now because the user-facing contract is already
changing; carrying both names would complicate docs and snapshots without
solving the source-attr mismatch.

### 5. Diagnostic wording is fully specified

**Choice:** Keep `no_panic` primary wording unchanged for its remaining macros:
```
`<macro>` will abort the process on execution
```
where `<macro>` is `panic!`, `unimplemented!`, or `unreachable!`.

Add `no_todo` primary wording:
```
`todo!()` marks unfinished production code
```

Add `no_todo` metadata:
- principle: `Fail Fast Correctly`
- guide section: `Error Handling / Panics`
- domain group: `safety`
- guidance: `replace temporary stubs with implemented code or an explicit error path before release`
- example: `return Err(Error::not_ready());`

Rename the companion metadata entry to `unjustified_no_todo_allow` with help:
```
Tiger Style (Documented Todo Escape): reserve `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` for temporary `todo!()` stubs with an adjacent `//` justification comment Example: // justified stub while API lands
#[allow(tigerstyle::no_todo)] fn stub() { todo!() }
```
and note:
```
see Tiger Style guide: Error Handling / Panics
```

The companion lint's primary messages become:
- missing comment:
  `` `#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` needs an adjacent `//` justification comment and may only wrap temporary `todo!()` stubs ``
- wrong macro:
  `` `#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` is reserved for temporary `todo!()` stubs, not `<macro>` ``
- orphaned attr:
  `` `#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` no longer covers any `todo!()` stub; remove it ``

**Rationale:** The repo's design-stage review expects exact user-visible
contracts, not approximate examples.

**Alternative:** Reuse the old "panic escape" principle/help text. Rejected
because the split is supposed to make the contract honest and self-describing.

### 6. Verification proves behavior, inventory, docs, and downstream blast radius

**Choice:** Verification must cover four surfaces:
1. lint routing (`todo!()` vs other panic macros)
2. `no_todo` allow hygiene
3. metadata/inventory publication
4. downstream migration docs and real consumer usage inventory

**Implementation:**
- unit tests for routing helpers and config/inventory wiring
- ignored UI fixtures with explicit filenames:
  - `ui_tests/no_todo.rs` for direct `todo!()` routing
  - `ui_tests/no_todo_legacy_no_panic_allow.rs` for both legacy `no_panic`
    spellings that no longer suppress `todo!()`
  - `ui_tests/no_todo_allow.rs` for justified/unjustified bare and tool-scoped
    `no_todo` attrs
  - `ui_tests/no_todo_orphan_allow.rs` for wrong-macro, orphaned, and
    `cfg_attr`-expanded audit cases
- refreshed diagnostic metadata snapshot artifact plus a saved artifact that
  prints the full snapshot contents
- a saved docs grep artifact over `README.md`, `docs/downstream-rollout.md`, and
  `templates/dylint.toml`
- a saved downstream probe artifact enumerating current `allow(no_panic)` /
  `allow(tigerstyle::no_panic)` uses in Aspen so review can compare docs against
  real migration targets

## Risks / Trade-offs

**[Downstream rename churn]** → Consumers with existing justified stubs must
rename attrs and possibly config keys. Mitigation: document both bare and
canonical spellings explicitly and save a real Aspen usage inventory artifact.

**[Diagnostic churn]** → New lint names and help text will refresh many UI
snapshots and metadata fixtures. Mitigation: make wording exact in this design
so reviewers can treat churn as contract-following, not accidental drift.

**[Bare-source ambiguity]** → Bare `allow(no_panic)` exists in live Aspen code,
while the canonical docs prefer tool-scoped attrs. Mitigation: rollout guidance
must mention both source spellings during migration even though canonical docs
still prefer `tigerstyle::...` on supported nightly toolchains.
