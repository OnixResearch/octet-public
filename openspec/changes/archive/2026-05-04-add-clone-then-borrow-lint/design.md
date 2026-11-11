## Context

The broad external-source incorporation work selected clone avoidance as an imported Rust idiom because unnecessary clones degrade performance and often obscure ownership. A broad "all suspicious clones" lint would be noisy: `clone()` is sometimes the clearest way to create an independently owned value, detach from a lock guard, or cross a thread/task boundary.

This child OpenSpec intentionally scopes the first candidate to immediate borrow-of-clone expressions, where the clone result is not observed as owned state before it is borrowed.

## Goals / Non-Goals

**Goals:**

- Warn on direct shared borrows of clone results, for example `foo(&name.clone())`.
- Keep the lint advisory-first while fixtures establish precision.
- Provide review-required suggestions that explain the safer ownership alternatives.
- Preserve clear false-positive boundaries for ownership-consuming, stored, independently mutated, and thread/lifetime-detaching clones.

**Non-Goals:**

- Do not attempt global borrow-checker reasoning in the first implementation.
- Do not warn on every `x.clone()` passed by value; owned arguments may be intentional.
- Do not rewrite multi-statement clone lifetimes without human review.
- Do not hard-gate clone avoidance until downstream inventory shows acceptable precision.

## Decisions

### 1. Start with immediate borrow-of-clone syntax

**Choice:** The first implementation should inspect expressions where a shared borrow directly wraps a clone call, such as `&value.clone()` or `&(value.clone())`.

**Rationale:** This shape has a clear ownership contradiction: code pays for an owned clone but immediately exposes only a borrow. It is easier to fixture than multi-statement ownership analysis.

**Alternative:** Detect `let tmp = value.clone(); foo(&tmp);`. Rejected for the first slice because the temporary may intentionally extend lifetime, detach from a lock/borrow, or be mutated later.

**Implementation:** Add a late lint pass under `src/performance/` that recognizes `ExprKind::AddrOf(_, Mutability::Not, clone_call)` and confirms the callee is the inherent or trait `Clone::clone` method before emitting.

### 2. Suggestions must be review-required

**Choice:** Emit a concrete but `MaybeIncorrect` suggestion when the original receiver span is available.

**Rationale:** Replacing `&value.clone()` with `&value` is often correct, but type coercions, receiver autoderef, or adapter methods may make `.as_ref()` / `.as_slice()` clearer. Human review preserves safety.

**Alternative:** Mark fixes `MachineApplicable`. Rejected until UI fixtures prove no type-adjustment surprises.

### 3. Keep ownership-detaching clones out of scope

**Choice:** The lint must not warn on clones that are moved, stored, sent across threads/tasks, or returned.

**Rationale:** Those clones may be real ownership boundaries and are not represented by the direct borrow-of-clone pattern.

**Implementation:** Initial syntax scope naturally skips these cases; future expansions must add explicit false-positive fixtures before implementation.

## Risks / Trade-offs

**Type coercion surprise** → Keep suggestions review-required and provide guidance alternatives rather than unconditional autofix.

**Macro/source-span noise** → Skip expansion-generated clone calls unless a future fixture proves useful macro behavior.

**False sense of complete clone analysis** → Document that `clone_then_borrow` is a narrow lint, not a replacement for review of broader clone-to-satisfy-borrow-checker patterns.
