## Context

Rust Design Patterns recommends accepting borrowed argument types such as `&str`, `&[T]`, and `&Path` instead of `&String`, `&Vec<T>`, and `&PathBuf` when ownership and owned-container APIs are not needed. Tigerstyle already values explicit, simple interfaces and passing large data by reference; this lint makes that idiom reviewable without requiring a broad style catalog import.

## Goals / Non-Goals

**Goals:**
- Flag obvious borrowed-owned-container parameter types in function and method signatures.
- Suggest the more general borrowed type where it is mechanically clear.
- Avoid trait- dictated signatures and FFI/extern ABI surfaces.
- Require fixture evidence before default hard-gate promotion.

**Non-Goals:**
- Inferring whether a body truly needs owned-container-specific APIs.
- Rewriting call sites or changing ownership semantics automatically.
- Flagging return types, struct fields, or generic trait bounds in the first slice.
- Enforcing this lint as deny-by-default in the first implementation.

## Decisions

### 1. Start with narrow signature detection

**Choice:** Inspect function/method parameter types for references to owned standard containers: `&String`, `&Vec<T>`, and `&PathBuf`.

**Rationale:** These cases have clear preferred forms (`&str`, `&[T]`, `&Path`) and align with existing Tigerstyle interface guidance.

**Alternative:** Implement a broad API-polymorphism lint that reasons about every owned type. Rejected because it would be noisy and hard to prove with finite fixtures.

**Implementation:** A future late lint can inspect `hir::FnDecl` inputs and type paths, skip generated/test/extern/trait-dictated contexts, and emit advisory diagnostics with MaybeIncorrect suggestions.

### 2. Keep trait and ABI boundaries out of scope

**Choice:** Do not flag trait declarations, trait impl methods, extern functions, or ABI boundary callbacks in the first slice.

**Rationale:** Those signatures are often dictated by external contracts; false positives would undermine trust in the lint.

**Alternative:** Flag them with allow suggestions. Rejected until downstream inventory proves useful signal.

### 3. Fixture gate before hard promotion

**Choice:** Require compliant, noncompliant, and false-positive-boundary UI fixtures before the lint can become default warn or deny.

**Rationale:** The parent OpenSpec requires pattern-derived lints to prove low false-positive behavior before hard gating.

## Risks / Trade-offs

**Type alias opacity** → The first implementation may miss aliases such as `type Name = String`; document as out of first scope unless HIR type resolution makes it cheap and reliable.

**Body-specific need for owned APIs** → A function may call `String`-specific methods. Start advisory with MaybeIncorrect suggestions and avoid machine-applicable fixes.

**Stable/public crate attr churn** → Keep bare lint-name configuration compatible with existing Tigerstyle lint naming rules and avoid requiring new tool-scoped attributes.
