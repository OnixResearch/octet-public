## Context

Tigerstyle incorporated Rust pattern guidance that cautions against `Deref` polymorphism: `Deref` should not be used to simulate inheritance or make a wrapper masquerade as another domain object. The first lint slice should be narrow enough to avoid flagging common Rust idioms like string/slice borrowed views.

## Goals / Non-Goals

**Goals:**

- Detect public local wrappers that implement `Deref` to nominal target types.
- Encourage explicit accessor methods such as `as_inner`, `inner`, or domain-specific methods.
- Preserve accepted `Deref<Target = str>` and `Deref<Target = [T]>` patterns.
- Keep rollout advisory and reviewable.

**Non-Goals:**

- Do not infer all smart-pointer correctness properties.
- Do not flag private wrappers used only inside a module.
- Do not inspect macro-generated impls.
- Do not emit automated rewrites; replacing public `Deref` is an API design decision.

## Decisions

### 1. Structure-domain advisory lint

**Choice:** Implement `deref_polymorphism` as a structure lint with default level `warn`.

**Rationale:** The issue is API shape: a public type's method surface expands implicitly through `Deref`.

**Alternative:** Put it in safety. Rejected because the core risk is unclear interface inheritance rather than immediate memory unsafety.

### 2. Nominal-target first slice

**Choice:** Warn when a public local self type implements `Deref` and its associated `Target` is a path/nominal type. Accept `str` and slice targets.

**Rationale:** Nominal targets are the highest-signal inheritance-like cases. `str` and slices are common borrowed-view idioms.

**Alternative:** Warn on every public `Deref`. Rejected as too noisy for an advisory first pass.

### 3. Diagnostic-only output

**Choice:** Emit guidance without structured suggestions.

**Rationale:** Replacing `Deref` may require accessor naming, API migration, and downstream compatibility review.

## Risks / Trade-offs

**False positives for smart pointers** → Mitigated by advisory severity and by focusing on local public wrappers with explicit UI accepted cases.

**Source-shape fragility** → Mitigated by using HIR trait information for `Deref` and source snippets only to classify the `Target` declaration conservatively.
