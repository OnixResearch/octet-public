## Context

The high-noise lint promotion playbook names five families that stay
inventory-first because of false-positive volume. Aspen phase-2 widening
confirmed the noise: `aspen-coordination` alone had 21 `ambient_clock`,
4 `no_panic`, and additional `no_unwrap` findings. Precision improvements
should reduce noise enough that these families become promotable.

## Goals / Non-Goals

**Goals:** Reduce false positive rate in the five named families without
weakening detection of real bugs.

**Non-Goals:** Eliminating all false positives. Some findings are
genuinely noisy because the underlying code pattern is debatable — those
belong in the narrow-allow workflow, not in lint suppression.

## Decisions

### 1. Test-context suppression for `no_unwrap` and `no_panic`

**Choice:** Both lints skip `#[test]` functions and `#[cfg(test)]` modules.

**Rationale:** Test code uses `.unwrap()` and `panic!` legitimately for
test assertions and setup. These are already excluded by spec for other
lints like `assertion_density`.

**Implementation:** Check HIR attrs plus enclosing-item source fallback for
`#[test]`, `#[cfg(test)]`, and inner `#![cfg(test)]`. Keep explicit UI proof
for `#[cfg(not(test))]` so suppression does not broaden back into production
paths.

### 2. Compile-time-known unwrap suppression

**Choice:** `no_unwrap` skips `.unwrap()` on expressions that are
provably `Some`/`Ok` from literal construction (e.g., string `.parse()`
on a literal, `Some(x).unwrap()`).

**Rationale:** These are idiomatic Rust for const initialization and
zero-cost at runtime. Flagging them adds noise without catching bugs.

**Implementation:** Check if the receiver of `.unwrap()` is `Some(...)`,
`Ok(...)`, or a string-literal `parse()` targeting a finite supported set of
standard types whose literal value can be validated cheaply at lint time:
primitive bool/int/float/char, `String`, and standard network address types
(`IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`,
`SocketAddrV6`). Invalid literal parses and analogous `.expect()` calls keep
warning, with explicit negative UI proof for those boundaries.

### 2b. Same-branch guard suppression for `no_unwrap`

**Choice:** `no_unwrap` skips `.unwrap()` only when the same branch is already
logically dominated by `.is_some()` or `.is_ok()` on the same value.

**Rationale:** This is a common Rust pattern for narrowing `Option` / `Result`
state before using the value. Flagging it adds noise, but only when the guard
really implies the unwrap is safe.

**Implementation:** Match the condition structure rather than searching raw
source snippets. `&&` may carry the guard into the branch, while `||` only
counts when both sides imply the same guard. Keep UI proof for both the silent
same-branch `.unwrap()` case and the still-warning `.expect()` / unsafe-branch
cases.

### 3. Drop-impl suppression for `ignored_result`

**Choice:** Skip `let _ = expr` inside `Drop::drop` implementations.

**Rationale:** `Drop::drop` cannot return `Result`. Ignoring errors
during cleanup is the only option. The error would have to be logged
or swallowed regardless.

**Implementation:** Check if the enclosing function is a `Drop::drop`
impl method, and match the trait through `tcx.lang_items().drop_trait()`
rather than by trait-name text.

### 4. Exhaustive-match unreachable suppression for `no_panic`

**Choice:** Skip `unreachable!()` in a trailing wildcard arm when earlier
arms explicitly cover every variant of a local enum.

**Rationale:** After listing all enum variants explicitly, a trailing
`_ => unreachable!()` is defensive programming, not a liveness hazard.

**Implementation:** In a late pass, inspect the enclosing `match`, require
that the `unreachable!()` is the direct body of the last wildcard arm,
collect concrete variant arms, and suppress only when they cover the full
local enum variant set. Do not suppress external or `#[non_exhaustive]`
enums.

**Alternative:** Only suppress when the compiler can prove exhaustiveness
— rejected because rustc's exhaustiveness check happens before our lint
pass and we'd need to duplicate that logic anyway.

### 5. Broader bounds-check recognition for `unchecked_narrowing`

**Choice:** Recognize `x.min(T::MAX as u64) as T` and
`T::try_from(x).unwrap_or(T::MAX)` as bounded patterns.

**Rationale:** These are common bounded-narrowing idioms that don't need
a preceding `assert!`.

**Implementation:** Pattern-match on the expression structure surrounding
the cast or conversion.

### 6. Justified `todo!()` suppression for `no_panic`

**Choice:** Suppress `todo!()` only when the call is inside a function or
block explicitly annotated with `#[allow(tigerstyle::no_panic)]` and the
attribute has a justification comment on the same line or the immediately
preceding non-empty line.

**Rationale:** Stub-in-progress escapes should stay narrow and auditable.
A loose comment rule would turn `todo!()` into an easy blanket opt-out.

**Implementation:** Reuse the repo's existing justification-comment shape:
only `// ...` comments count, with no blank line between comment and
attribute. Doc comments and block comments do not count. Standalone UI
fixtures that exercise the tool-scoped attr must register the tool; this is a
change-contract-driven exception to the repo's usual preference for bare lint
names in standalone fixtures.

### 7. Shutdown-send suppression for `ignored_result`

**Choice:** Limit fire-and-forget send suppression to `let _ = tx.send(msg);`
in functions or methods named exactly `shutdown` or `close`, and only when
`tx` has type `std::sync::mpsc::Sender<_>` or `tokio::sync::mpsc::Sender<_>`.

**Rationale:** Receiver disappearance is expected during teardown, but the
same `send` pattern is risky in ordinary request or replication paths.
A narrow type-and-name rule keeps the exception reviewable.

**Implementation:** Match the ignored expression as a `send` method call on
those sender types, including the awaited `send(...).await` shape by unwrapping
Rust's await desugar before checking the receiver type and enclosing function
or impl item name. Resolve Tokio sender identity through its concrete
definition path rather than assuming the public re-export spelling appears in
`get_def_path`. `Drop::drop` remains a separate rule rather than being folded
into this one. Keep the existing diagnostic/help text unchanged for still-
warning ignored-result cases so ordinary `let _ = expr` and non-teardown
`send` calls continue steering users toward `let _: () = expr?;` or explicit
error handling.

### 8. Same-block `assert!` recognition for `unchecked_narrowing`

**Choice:** Treat earlier `assert!(value <= T::MAX as WIDE)` statements in
the same block as a valid bound for a later `value as T` cast.

**Rationale:** Real code often separates validation from the cast with
logging or small helper statements. Requiring the assert to be immediately
adjacent adds noise without improving safety.

**Implementation:** Walk the enclosing block, inspect only statements that
textually precede the cast, and match the same binding / expression identity
plus the `<= TARGET::MAX as WIDE` bound shape. Do not fall back to whole-block
text search or raw source-text equality that could confuse shadowed values.

### 9. `tokio::select!` loop recognition for `unbounded_loop`

**Choice:** Treat `loop { tokio::select! { ... } }` and `loop { select! { ... } }`
(imported from Tokio) as bounded event-loop patterns.

**Rationale:** These loops wait on external readiness and are normal control
flow in async services. The precision goal is to stop flagging the common
Tokio case, not to bless arbitrary macros named `select!`.

**Implementation:** Detect the loop-body macro callsite from
`expr.span.source_callsite()` and confirm Tokio origin through macro-backtrace
metadata. Keep the rule narrow; other event-loop macros stay out of scope for
this change.

### 10. Bounded receiver recognition for `unbounded_loop`

**Choice:** Treat `while let Some(msg) = rx.recv().await` as bounded only
when `rx` has type `tokio::sync::mpsc::Receiver<_>`. Do not suppress
`tokio::sync::mpsc::UnboundedReceiver<_>`.

**Rationale:** The bounded Tokio receiver has built-in capacity limits and
is the main downstream pattern we care about. Collapsing bounded and
unbounded receiver types would hide real liveness risks.

**Implementation:** Resolve the receiver type of `recv().await` through HIR
and typeck results, matching Tokio's concrete bounded / unbounded definition
paths instead of just the public re-export spelling. The positive and negative
UI fixtures should use a small fixture-local Cargo crate that provides Tokio
externs to the standalone UI harness.

### 11. Async UI harness parity

**Choice:** Keep the standalone UI harness compiling every fixture with
`--edition=2024`.

**Rationale:** Pending async fixtures (`tokio::select!`, `recv().await`) must
compile under the repo's real edition. Falling back to rustc's default edition
would create false failures unrelated to the lint logic.

**Implementation:** Preserve `--edition=2024` in `tests/ui.rs`, and let
fixture-local Cargo support add Tokio externs without changing the direct
standalone rustc edition used for the actual linted fixture.

## Verification strategy

- Keep explicit positive UI proof that `#[cfg(test)]` and inner
  `#![cfg(test)]` suppression works for both `no_unwrap` and `no_panic`, plus
  negative UI proof that `#[cfg(not(test))]` production code still warns.
- Keep explicit positive UI proof for compile-time-known `Some(...)`,
  `Ok(...)`, const/static-style initializer `.unwrap()` use, and valid
  literal-string `.parse().unwrap()` exemptions, plus negative proof that
  invalid literal parses and analogous `.expect()` calls still warn.
- Keep explicit positive UI proof for `unchecked_narrowing` on
  `min(... as ...) as target`, `try_from(...).unwrap_or(T::MAX)`, and earlier
  same-block `assert!(...)`, plus negative proof for unrelated, shadowed,
  mismatched, and later asserts.
- Keep explicit negative UI proof that unsafe `else` branches and other
  non-dominated paths still warn for guarded `no_unwrap` cases.
- Keep explicit negative UI proof that external enums,
  `#[non_exhaustive]` local enums, and indirect wildcard-arm panics still warn.
- Keep explicit positive UI proof for both exhaustive local-enum wildcard
  `unreachable!()` suppression and function-scoped / block-scoped justified
  `todo!()` allows, plus negative UI proof that unjustified `todo!()` still
  warns when the adjacent `//` comment rule is not satisfied.
- Keep explicit positive UI proof for `ignored_result` both inside
  `Drop::drop`, for `std::sync::mpsc::Sender<_>` in `shutdown`, and for
  `tokio::sync::mpsc::Sender<_>` in `close`, plus negative UI proof that
  ordinary sends and exact-name `try_send` teardown calls still warn.
- Keep explicit regression proof that existing visible-counter and
  counter-break `unbounded_loop` cases stay silent.
- Keep explicit negative UI proof that `while let Some(_) = iter.next()`
  without a limit check still warns, so the Tokio-specific precision work does
  not broaden into generic iterator loops.
- Keep explicit positive UI proof for both fully qualified `tokio::select!`
  and imported Tokio `select!`, plus negative proof that non-Tokio `select!`,
  bare async loops, and `UnboundedReceiver<_>` loops still warn for
  `unbounded_loop`.
- Keep explicit diagnostic regression proof that ordinary `ignored_result`
  warnings, including non-teardown `send` calls, still emit the existing help
  text about `let _: () = expr?;` or explicit error handling.
- Keep explicit async UI proof that plain `.await` outside a user-written loop
  stays silent so desugared polling loops do not reintroduce false positives.

## Risks / Trade-offs

**[Over-suppression]** → Each new exception is a potential gap. Mitigation:
every suppression rule gets a UI test fixture proving both the suppression
and the continued detection of the real-bug case.

**[Downstream validation]** → Must re-run against Aspen to confirm noise
reduction without losing real findings. Mitigation: save before/after
finding counts per affected lint family as evidence, treat any increase in
false positives or any lost real finding as a regression, and summarize the
before/after comparison in the reviewer-visible artifact
`openspec/changes/lint-precision-hardening/evidence/aspen-comparison.md`.
