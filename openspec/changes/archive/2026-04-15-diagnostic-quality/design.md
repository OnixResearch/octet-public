## Context

Phase 1 and Phase 2 of `diagnostic-quality` already delivered the shared
`src/diagnostics.rs` metadata inventory, the canonical
`Tiger Style (...)` / `see Tiger Style guide: ...` wording, and one shared
emission wrapper. Phase 3 exposed a namespace mismatch that the original design
under-specified:

- tigerstyle still declares plain `declare_lint!` statics
- per-lint config currently feeds bare names through `sess.opts.lint_opts`
- proposal, README, and the domain-group plan all describe a canonical
  `tigerstyle::...` public namespace
- rustc only resolves tool-scoped group attrs like
  `#![allow(tigerstyle::safety)]` against true tool-lint registrations

With plain lint statics, `#![allow(tigerstyle::safety)]` and
`#![deny(tigerstyle::naming)]` do not work. A naive switch to
`declare_tool_lint!` makes those group attrs viable, but it breaks the current
bare-name config path with `unknown lint tool: tigerstyle` when rustc tries to
resolve `sess.opts.lint_opts` before any explicit tool registration exists.

This change therefore needs one coherent model:

- canonical consumer-visible names are true tool-lint names
- domain groups are true tool-scoped groups
- shared diagnostic metadata stays the source of truth
- bare `dylint.toml` keys and repo-local bare fixture paths keep working during
  the migration

The existing standalone UI harness already compiles fixtures with
`--edition=2024` and prints each fixture name. Canonical tool-scoped fixture
attrs still need `#![feature(register_tool)]` and
`#![register_tool(tigerstyle)]` when compiled as standalone raw-rustc crates.

## Goals / Non-Goals

**Goals:**
- Keep every emitted diagnostic on one canonical shared warning/help/note path
- Make every documented tigerstyle lint and domain group a true
  `tigerstyle::...` tool-lint identity
- Preserve bare `dylint.toml` keys (`disabled_lints`, `lint_levels`) and
  existing bare compatibility fixtures during the migration
- Prove `declare_tool_lint!` plus the bare compatibility-alias path on one
  representative lint under the repo's Dylint harness before bulk rollout
- Prove both canonical tool-group attrs and bare config compatibility with
  deterministic snapshots and UI fixtures

**Non-Goals:**
- Stable-toolchain support for tool-scoped attrs
- New lint logic beyond the namespace/config migration already needed for
  diagnostic-quality
- A runtime function-pointer registration table or cloned `Lint` descriptors

## Decisions

### 1. Canonical diagnostic wording stays centralized

**Choice:** Every lint continues to emit:
1. one warning message describing the violation
2. one help line in the exact form
   `Tiger Style (<principle>): <guidance>`; when a lint has a concrete approved
   pattern, `<guidance>` includes an appended `Example: <approved pattern>`
   clause in the same help line
3. one note in the exact form `see Tiger Style guide: <section>`

**Rationale:** Phase 1 and Phase 2 already settled the wording contract. The
namespace migration must not reopen help/note drift.

**Implementation:** Keep `src/diagnostics.rs` as the only module that renders
Tiger Style help/note text. Lint modules provide only their primary warning
message plus the bare inventory key used to look up metadata.

**Alternative:** Let the tool-lint migration rewrite help/note text in each
lint module. Rejected because it reintroduces exactly the drift this change was
created to remove.

### 2. One metadata inventory remains the source of truth

**Choice:** Keep one `LintDiagnosticMeta` inventory in `src/diagnostics.rs`
covering the original 25 lints plus the companion safety lint
`unjustified_no_panic_allow`. Its `domain_group` field stores canonical
tool-scoped group names like `tigerstyle::safety`.

**Rationale:** The same table already drives diagnostic wording. It should also
own canonical group names and inventory-derived registration checks so the tool
migration does not duplicate the lint matrix in `src/lib.rs`, docs, and tests.

**Implementation:** Continue snapshot-testing the full metadata table and
requiring exact name parity with `LINT_INVENTORY`.

### 3. Full migration is gated by a single-lint Dylint spike

**Choice:** Phase 3 starts with one representative lint spike before any
bulk conversion of the 25 pre-existing lints.

**Rationale:** Two load-bearing assumptions remain unproven in this repo's
actual Dylint context:
- `declare_tool_lint!(pub tigerstyle::...)` must still register and emit
  correctly through the existing Dylint harness
- a one-member bare compatibility alias must resolve the current bare config /
  lint-level path without `unknown lint tool: tigerstyle`

Rustc source suggests the alias path may work through `lint_groups`, but the
repo's own napkin rule is clear: do not assume group registration alone is
enough. This change therefore treats the mechanism as a migration precondition,
not as an established fact.

**Implementation:**
- use `no_unwrap` as the representative spike lint because the repo already has
  focused allow/warn/deny/disabled fixtures for it
- convert only that lint to `declare_tool_lint!(pub tigerstyle::no_unwrap)` in
  the first spike
- register one-member bare compatibility only for `no_unwrap` in that spike
- prove all of these before converting the remaining 24 lints:
  - the Dylint library still loads and emits `tigerstyle::no_unwrap`
  - canonical tool-scoped attrs resolve in a standalone UI fixture with
    `#![feature(register_tool)]` and `#![register_tool(tigerstyle)]`
  - bare `disabled_lints`, bare `[tigerstyle.lint_levels]`, and the current
    bare lint-level path still resolve through the alias

**Alternative:** Convert all pre-existing lints first and debug the breakage afterward.
Rejected because the migration has too many moving parts to unwind safely if the
foundational assumption is wrong.

### 4. Every documented tigerstyle lint becomes a true tool lint

**Choice:** Convert every consumer-facing lint static from `declare_lint!` to
`declare_tool_lint!(pub tigerstyle::...)`.

**Rationale:** Tool-scoped attrs and tool-scoped groups are the documented
public API. Rustc only treats them natively as such when the lint statics are
true tool lints. This makes the public API and the compiler model match.

**Implementation:**
- keep `LINT_INVENTORY` keyed by the existing bare logical names like
  `no_unwrap`
- keep the original static lint references in the inventory
- add helpers that derive canonical tool names (`tigerstyle::no_unwrap`) from
  those bare keys for docs, snapshots, and registration checks
- do not introduce plain-lint shadow copies of the same lint

**Alternative:** Keep plain `declare_lint!` statics and only register a
`tigerstyle::safety` group name. Rejected because tool-scoped group attrs do
not resolve correctly in that mixed model.

### 5. Bare compatibility aliases preserve config and existing bare fixture paths

**Choice:** Register one-member bare compatibility aliases for each enabled
lint, derived from the same inventory and pointing at the original static
`LintId`.

**Rationale:** rustc command-line lint resolution appears to fall back from
`by_name` to `lint_groups`, so a bare alias like `no_unwrap` may be able to
resolve to the real `tigerstyle::no_unwrap` lint via a one-member group even
when the crate has not registered the `tigerstyle` tool name. That is still a
hypothesis until the Phase 3 spike proves it under this repo's Dylint harness.
If the spike confirms it, one-member aliases give the smallest bridge from bare
config keys to canonical tool lints without pretending that the public API is
still plain-lint based.

**Implementation:**
- first prove the one-member alias path on `no_unwrap` in the Phase 3 spike
- only after that spike passes, register a one-member bare alias for each
  enabled inventory entry derived from the bare inventory key
- treat those aliases as compatibility shims, not the preferred documented API
- snapshot-test the bare alias map so every bare key resolves to the intended
  tool lint
- do not register bare compatibility aliases for disabled lints

**Alternative:** Build a wrapper that injects only tool-scoped command-line
lint flags like `-D tigerstyle::no_unwrap`. Rejected as the primary fix because
ordinary crates still hit `unknown lint tool: tigerstyle` without explicit tool
registration, so wrapper-only flag injection does not solve the base problem.

### 6. Domain groups stay canonical and tool-scoped

**Choice:** Register these canonical groups from inventory-derived lists of the
original static lint IDs:
- `tigerstyle::safety` (12 lints)
- `tigerstyle::naming` (4 lints)
- `tigerstyle::structure` (6 lints)
- `tigerstyle::purity` (3 lints)
- `tigerstyle::concurrency` (1 lint)

**Rationale:** The public API should expose one tool-scoped domain namespace.
Bare compatibility aliases exist only to preserve config and legacy fixture
paths; they are not a second public group API.

**Implementation:**
- derive membership from the same metadata inventory used by diagnostics
- omit disabled lints from canonical groups as well as bare aliases
- snapshot-test the five exact canonical group names and ordered memberships

**Alternative:** Hand-maintain a second group matrix in `src/lib.rs`. Rejected
because it duplicates the metadata inventory and makes drift more likely.

### 7. `no_panic` needs a companion lint for escape-hatch misuse

**Choice:** Keep `no_panic` as a true tool lint, but add a separate safety lint
`unjustified_no_panic_allow` that audits `#[allow(tigerstyle::no_panic)]`
escapes.

**Rationale:** Rustc `#[allow(...)]` is binary. Once `no_panic` becomes a real
tool lint, `#[allow(tigerstyle::no_panic)]` suppresses the lint before any
custom justification logic can decide whether a `todo!()` stub was locally
justified. The most correct fix is therefore not to fight rustc's lint-level
semantics, but to keep `no_panic` as the panic detector and make the escape
hatch itself reviewable through a second lint.

**Implementation:**
- `no_panic` continues to detect `panic!()`, `todo!()`, `unimplemented!()`, and
  `unreachable!()` when no escape hatch is in play
- `unjustified_no_panic_allow` warns when `#[allow(tigerstyle::no_panic)]` is
  missing the adjacent `//` justification comment required for temporary
  `todo!()` stubs
- the same companion lint also warns when that escape hatch is used around
  `panic!()`, `unimplemented!()`, or `unreachable!()` instead of a temporary
  `todo!()` stub

**Alternative:** Rely on `#[allow(tigerstyle::no_panic)]` itself to be
conditional. Rejected because rustc does not provide such a hook.

### 8. Bare config keys stay stable while canonical docs/examples use tool names

**Choice:** Keep bare lint names in `dylint.toml` while standardizing README,
downstream docs, emitted examples, and canonical source-attr fixtures on
`tigerstyle::...` names.

**Rationale:** Bare config keys are already part of the consumer contract and
are convenient to type. The migration should make the public lint identity more
correct without forcing a config-key rename.

**Implementation:**
- `disabled_lints` continues to suppress registration by bare key before any
  pass or group alias is registered
- `lint_levels` continues to feed bare keys through the existing config path,
  which now resolves via the compatibility aliases instead of plain lint
  registrations
- examples that show source attrs in docs, diagnostics, or UI fixtures use the
  canonical `tigerstyle::...` form unless the fixture is explicitly proving the
  bare compatibility path

**Alternative:** Rename `dylint.toml` keys to `tigerstyle::no_unwrap`. Rejected
because it creates needless migration churn for users while providing little
benefit over compatibility aliases.

### 9. Standalone UI fixtures separate canonical and compatibility paths

**Choice:** Use two distinct fixture styles:
- canonical tool-scoped fixtures for domain groups and documented tool attrs
- existing bare per-lint fixtures where the point is compatibility or
  config-driven behavior

**Rationale:** The UI harness needs to prove both the public tool-scoped API and
the compatibility bridge that keeps bare config keys working.

**Implementation:**
- canonical tool-group fixtures include `#![feature(register_tool)]`,
  `#![register_tool(tigerstyle)]`, and the existing `--edition=2024` UI path
- existing bare allow-by-default and config fixtures remain bare when they are
  intentionally exercising compatibility aliases
- fixture-set artifacts distinguish canonical tool-group fixtures from bare
  compatibility/config fixtures

### 10. Verification covers wording, groups, aliases, config compatibility, and the `no_panic` escape hatch

**Choice:** Verification must cover all of these layers:
1. metadata completeness against `LINT_INVENTORY`
2. full metadata snapshot including principle, guide section, canonical domain
   group, guidance, example mode, and example text
3. canonical tool-group membership snapshot
4. bare compatibility-alias snapshot mapping every bare lint key to its tool
   lint target
5. shared-helper enforcement that rejects ad-hoc help/note text outside
   `src/diagnostics.rs`
6. standalone UI fixtures proving `#![allow(tigerstyle::safety)]` suppresses
   safety lints while a naming lint still warns, plus
   `#![deny(tigerstyle::naming)]`
7. config/UI proof that bare `disabled_lints` and bare `[tigerstyle.lint_levels]`
   keys still work after the tool-lint migration
8. dedicated UI proof that justified `#[allow(tigerstyle::no_panic)]`
   `todo!()` stubs stay quiet while unjustified or non-`todo!()` uses warn via
   `unjustified_no_panic_allow`
9. refreshed `.stderr` fixtures plus `cargo test --lib`,
   `cargo test --test ui -- --ignored --nocapture`, and `nix flake check`
10. downstream starter-rollout revalidation against
   `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` so the
   canonical Aspen consumer reference still works after the namespace change

**Rationale:** Passing UI `.stderr` files alone does not prove that the
canonical tool namespace is real or that the bare config contract survived the
migration.

## Lint Matrix

| Lint | Canonical lint name | Domain group | Principle | Guide section | Example in help |
| --- | --- | --- | --- | --- | --- |
| `compound_assertion` | `tigerstyle::compound_assertion` | `tigerstyle::safety` | Assertion Quality | Assertion Density | Required |
| `assertion_density` | `tigerstyle::assertion_density` | `tigerstyle::safety` | Assertion Density | Assertion Density | None |
| `unbounded_loop` | `tigerstyle::unbounded_loop` | `tigerstyle::safety` | Bounded Control Flow | Bounded Control Flow | Required |
| `ignored_result` | `tigerstyle::ignored_result` | `tigerstyle::safety` | Error Handling | Error Handling | Required |
| `unchecked_narrowing` | `tigerstyle::unchecked_narrowing` | `tigerstyle::safety` | Safe Narrowing | Safe Narrowing / Conversions | Required |
| `unchecked_division` | `tigerstyle::unchecked_division` | `tigerstyle::safety` | Explicit Division | Explicit Division | Required |
| `raw_arithmetic_overflow` | `tigerstyle::raw_arithmetic_overflow` | `tigerstyle::safety` | Overflow Protection | Overflow Protection | Required |
| `catch_all_on_enum` | `tigerstyle::catch_all_on_enum` | `tigerstyle::safety` | Exhaustive Matching | Exhaustive Matching | Required |
| `no_unwrap` | `tigerstyle::no_unwrap` | `tigerstyle::safety` | Error Handling | Error Handling | Required |
| `no_panic` | `tigerstyle::no_panic` | `tigerstyle::safety` | Fail Fast Correctly | Error Handling / Panics | Required |
| `unjustified_no_panic_allow` | `tigerstyle::unjustified_no_panic_allow` | `tigerstyle::safety` | Documented Panic Escape | Error Handling / Panics | Required |
| `no_recursion` | `tigerstyle::no_recursion` | `tigerstyle::safety` | Simple Control Flow | Simple Control Flow | Required |
| `bool_naming` | `tigerstyle::bool_naming` | `tigerstyle::naming` | Positive Predicates | Positive Predicates | Required |
| `acronym_style` | `tigerstyle::acronym_style` | `tigerstyle::naming` | Acronym Capitalization | Acronym Capitalization | Required |
| `numeric_units` | `tigerstyle::numeric_units` | `tigerstyle::naming` | Unit Suffixes | Unit Suffixes | Required |
| `float_for_currency` | `tigerstyle::float_for_currency` | `tigerstyle::naming` | Integer Currency | Integer Currency | Required |
| `function_length` | `tigerstyle::function_length` | `tigerstyle::structure` | Function Length | Function Length | None |
| `compound_condition` | `tigerstyle::compound_condition` | `tigerstyle::structure` | Condition Decomposition | Condition Decomposition | Required |
| `ambiguous_params` | `tigerstyle::ambiguous_params` | `tigerstyle::structure` | Explicit Interfaces | Explicit Interfaces | Required |
| `platform_dependent_cast` | `tigerstyle::platform_dependent_cast` | `tigerstyle::structure` | Explicit Types | Explicit Types | Required |
| `unjustified_allow` | `tigerstyle::unjustified_allow` | `tigerstyle::structure` | Documented Suppression | Documented Suppression | Required |
| `nested_conditionals` | `tigerstyle::nested_conditionals` | `tigerstyle::structure` | Simple Control Flow | Simple Control Flow | Required |
| `ambient_clock` | `tigerstyle::ambient_clock` | `tigerstyle::purity` | Functional Core | Functional Core / Imperative Shell | Required |
| `contradictory_time` | `tigerstyle::contradictory_time` | `tigerstyle::purity` | Functional Core | Functional Core / Imperative Shell | Required |
| `verified_purity` | `tigerstyle::verified_purity` | `tigerstyle::purity` | Verified Module Purity | Verified Module Purity | Required |
| `multi_lock_ordering` | `tigerstyle::multi_lock_ordering` | `tigerstyle::concurrency` | Lock Ordering | Lock Ordering | Required |

## Canonical Guidance and Examples

This appendix remains the source of truth for emitted `guidance` and
`example_text` strings. Examples that show an attr use the canonical
`tigerstyle::...` form.

### Safety

- `compound_assertion`
  - guidance: split compound assertions so the failing clause is obvious
  - example: `assert!(a > 0); assert!(b > 0);`
- `assertion_density`
  - guidance: add enough assertions to make local invariants visible near the logic they defend
  - example: none
- `unbounded_loop`
  - guidance: bound long-running loops with an exit condition, timeout, or bounded input
  - example: `while let Some(msg) = rx.recv().await { ... }`
- `ignored_result`
  - guidance: handle fallible results explicitly with `?`, `match`, or an intentional logged drop
  - example: `tx.send(msg)?;`
- `unchecked_narrowing`
  - guidance: narrow integers with checked conversions instead of `as`
  - example: `let small = u32::try_from(value)?;`
- `unchecked_division`
  - guidance: guard divisors before division so zero is impossible
  - example: `assert!(divisor != 0); let q = value / divisor;`
- `raw_arithmetic_overflow`
  - guidance: use checked or saturating arithmetic when overflow would change behavior
  - example: `let total = a.checked_add(b)?;`
- `catch_all_on_enum`
  - guidance: match enum variants explicitly so new variants cannot slip through `_`
  - example: `match state { State::Open => ..., State::Closed => ... }`
- `no_unwrap`
  - guidance: propagate or handle absence explicitly instead of panicking with `unwrap` or `expect`
  - example: `let value = maybe?;`
- `no_panic`
  - guidance: return errors or prove exhaustiveness instead of panicking in production code
  - example: `return Err(Error::invalid_state());`
- `unjustified_no_panic_allow`
  - guidance: reserve `#[allow(tigerstyle::no_panic)]` for temporary `todo!()` stubs with an adjacent `//` justification comment
  - example: `// justified stub while API lands\n#[allow(tigerstyle::no_panic)] fn stub() { todo!() }`
- `no_recursion`
  - guidance: prefer explicit iteration or work queues over self-recursive control flow
  - example: `while let Some(item) = stack.pop() { ... }`

### Naming

- `bool_naming`
  - guidance: name booleans as predicates so call sites read like questions
  - example: `let is_ready = status == Ready;`
- `acronym_style`
  - guidance: capitalize acronyms like ordinary words in type and variant names
  - example: `RpcHandler`
- `numeric_units`
  - guidance: put the unit in the name of numeric quantities
  - example: `timeout_ms`
- `float_for_currency`
  - guidance: store money as scaled integers, not floats
  - example: `let price_micros: i64 = 125_000;`

### Structure

- `function_length`
  - guidance: split long functions into named helpers at invariant or phase boundaries
  - example: none
- `compound_condition`
  - guidance: break multi-part conditions into named predicates or nested guards
  - example: `if is_leader { if has_quorum { ... } }`
- `ambiguous_params`
  - guidance: replace same-type parameter clusters with named fields or an options struct
  - example: `fn connect(opts: ConnectOptions) -> ...`
- `platform_dependent_cast`
  - guidance: use explicit-width types at boundaries instead of `usize` or `isize` conversions
  - example: `let len: u32 = value.try_into()?;`
- `unjustified_allow`
  - guidance: every lint suppression needs a local reason tied to the code shape
  - example: `// Verified casts: Verus checks every narrowing in this module.\n#[allow(clippy::cast_possible_truncation)]`
- `nested_conditionals`
  - guidance: flatten control flow with guards, early returns, or helper predicates
  - example: `if !is_ready { return; }`

### Purity

- `ambient_clock`
  - guidance: pass time into core logic or isolate clock reads in a shell-boundary helper with a narrow allow reason
  - example: `#[allow(tigerstyle::ambient_clock, reason = "shell boundary clock read")] fn read_now() -> Instant { Instant::now() }`
- `contradictory_time`
  - guidance: choose one source of time for a code path and thread it through consistently
  - example: `fn expires_at(now: Instant, ttl: Duration) -> Instant { now + ttl }`
- `verified_purity`
  - guidance: keep verified modules free of runtime-only side effects and shell concerns
  - example: `pub fn next_state(input: Input) -> Output`

### Concurrency

- `multi_lock_ordering`
  - guidance: document and preserve one lock acquisition order across the codebase
  - example: `// Lock order: state -> peer -> metrics`

## Risks / Trade-offs

**[Unproven tool-lint compatibility path]** -> `declare_tool_lint!` and the
one-member bare alias plan may behave differently under Dylint than rustc's
source suggests. Mitigation: start with a single-lint `no_unwrap` spike and do
not generalize until it passes.

**[Tool-lint migration churn]** -> Lint declarations, group registration,
fixtures, README examples, and snapshots all move together. Mitigation: keep
one inventory-derived source of truth and require snapshots for both canonical
groups and bare compatibility aliases.

**[Compatibility alias drift]** -> Bare config or legacy fixtures could point at
the wrong canonical lint after the migration. Mitigation: add a deterministic
bare-alias snapshot and keep config fixtures in the verification set.

**[Downstream rollout drift]** -> The archived Aspen starter-rollout reference
may need docs or script updates after the namespace change. Mitigation:
revalidate against `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
before calling the change done.

**[Group fixture pitfalls]** -> Tool-scoped attrs fail without `register_tool`.
Mitigation: keep canonical group fixtures self-contained and spell out the
nightly `register_tool` boilerplate in docs and tests.
