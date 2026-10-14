## Context

Tigerstyle already catches several ways code hides control-flow or data-shape
risk behind convenient syntax: `no_unwrap`, `explicit_defaults`,
`unbounded_channel`, and `platform_dependent_cast`. It does not yet catch the
narrow pattern where fallible or optional state is forced back into an integer
slot with an extreme sentinel such as `u64::MAX`.

Recent consumer review exposed exactly that shape while building a string ID:
`u64::try_from(command_index).unwrap_or(u64::MAX)`. The bug pattern is not the
string formatting by itself; it is the silent conversion of failure into a
magic extreme value that can leak farther than intended.

## Goals / Non-Goals

**Goals:**
- Add one warn-by-default safety lint for extreme sentinel fallback on
  `Option` / `Result`
- Keep scope narrow enough for low-noise HIR/type-based matching
- Produce deterministic UI proof for positive and negative cases

**Non-Goals:**
- General magic-number detection
- Dataflow past the `unwrap_or` call into every later use
- Autofix generation
- Protocol-aware exemption logic beyond the existing allow/config paths

## Decisions

### 1. Match only `Option` / `Result` fallback combinators

**Choice:** Detect these forms only:
- `Option::unwrap_or(<sentinel>)`
- `Option::unwrap_or_else(|_| <sentinel>)`
- `Result::unwrap_or(<sentinel>)`
- `Result::unwrap_or_else(|_| <sentinel>)`

**Rationale:** The concrete review issue uses that shape, and these combinators
make the fallback intent explicit in HIR. Starting here avoids a broad,
textual "any match with a magic number" rule.

**Implementation:** Late pass. In `check_expr`, match `ExprKind::MethodCall`
for `unwrap_or` and `unwrap_or_else`. Use `cx.typeck_results().expr_ty(receiver)`
on the receiver expression, peel references, and require the ADT def to match
Rust's `Option` or `Result` lang-item identity before continuing.

### 2. Treat only integer extreme associated constants as sentinels in v1

**Choice:** Warn only when the fallback expression resolves to `<int>::MAX` or
`<int>::MIN` for one of `u8..u128`, `i8..i128`, `usize`, or `isize`, and that
constant's type matches the type returned by the `unwrap_or` /
`unwrap_or_else` call.

**Rationale:** `MAX` and `MIN` are the highest-signal fake-valid placeholders.
This keeps the rule narrow and avoids arguing about every `0`, `1`, or
project-local constant.

**Implementation:** Use resolved paths / type identity, not raw text search.
Treat the fallback as a candidate only when the tail expression is a direct
associated-const path that resolves to `MAX` or `MIN`; compare that resolved
const's parent integer type against the type returned by the `unwrap_or` /
`unwrap_or_else` call. Skip casts, local const aliases, or other transformed
expressions in the first pass.

### 3. Keep diagnostics manual and domain-directed

**Choice:** Pin the full user-visible contract in design.

**Warning message:** `extreme sentinel fallback masks absence or failure`

**Help text:** `Tiger Style (safety): branch explicitly, propagate the missing/error state, or emit tagged fallback text instead of <int>::MAX/<int>::MIN. Example: match u64::try_from(command_index) { Ok(i) => format!("recent_{i}"), Err(_) => "recent_missing_index".to_string() }`

**Note text:** `see Tiger Style guide: make invalid state explicit`

**Metadata contract:**
- lint name: `sentinel_fallback`
- canonical group: `safety`
- default level: `warn`
- structured suggestion tier: none in v1

**Representative `.stderr` shape:**
```text
warning: extreme sentinel fallback masks absence or failure
  --> ui_tests/sentinel_fallback.rs:4:20
   |
4  |     let id = format!("recent_{}", u64::try_from(command_index).unwrap_or(u64::MAX));
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: Tiger Style (safety): branch explicitly, propagate the missing/error state, or emit tagged fallback text instead of <int>::MAX/<int>::MIN. Example: match u64::try_from(command_index) { Ok(i) => format!("recent_{i}"), Err(_) => "recent_missing_index".to_string() }
   = note: see Tiger Style guide: make invalid state explicit
```

**Rationale:** The right fix depends on domain intent. Some callers should
return `Option`, some should `match`, and some should format a named fallback
like `recent_missing_index`.

### 4. Support closure-body sentinels without broad dataflow

**Choice:** Match both single-expression closures like `|_| u64::MAX` and block
closures whose tail expression is the sentinel, including side-effecting forms
such as `|err| { log(err); i64::MIN }`.

**Rationale:** Side effects do not make the sentinel any safer. If the closure's
returned value is still an extreme sentinel, the same hidden-state problem
remains.

**Implementation:** For `unwrap_or_else`, require the argument expression to be
an `ExprKind::Closure`. Read that closure's body id from the HIR closure node,
load the body through the current `TyCtxt` HIR body accessor, and inspect the
body value / block tail expression as the fallback candidate. For block
closures, intermediate statements do not suppress the lint; only the returned
final expression decides whether the fallback is an extreme sentinel.

## Verification strategy

- Positive UI case for `u64::try_from(command_index).unwrap_or(u64::MAX)` inside
  `format!`
- Positive UI case for `Option<u32>::unwrap_or(u32::MAX)`
- Positive UI case for a side-effecting `unwrap_or_else` closure that returns
  `i64::MIN`
- Negative UI case for explicit `match` / branching
- Negative UI case for `unwrap_or(0)` or another non-extreme default
- Negative UI case for `unwrap_or_else(|_| 0)`
- Negative UI case in supported test context

Verification commands:
- `nix develop -c bash -c 'cargo test --lib'`
- `nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'`

Saved evidence for archive review must include the full ignored UI transcript,
not only a pass/fail summary, so reviewers can see the exact rendered
`sentinel_fallback` diagnostic text in context.

## Risks / Trade-offs

**[False negatives from narrow scope]** → match expressions or casted sentinels
stay out of scope in v1. Mitigation: land narrow rule first, then expand only
with evidence.

**[Legitimate protocol sentinel use]** → some code intentionally reserves
`MAX` / `MIN`. Mitigation: keep lint warn-by-default, narrow to fallible
fallback combinators, and let existing allow/config paths handle reviewed
exceptions.
