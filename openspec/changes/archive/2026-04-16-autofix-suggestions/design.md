## Context

Dylint lints emit diagnostics through the rustc `Diagnostic` API, which
supports structured suggestions with `Applicability` levels. The
`span_suggestion` and `multipart_suggestion` methods let tigerstyle attach
reviewable edits directly to a lint diagnostic. `cargo fix` only auto-applies
`MachineApplicable` suggestions, so this change needs an explicit contract for
what counts as safe.

For this change, the supported automatic-application proof path is `cargo fix`
run inside the repo's nix devshell. `rustfix` is not a second required
acceptance path.

All rollout lints in this change are already registered as tool lints, so the
implementation does not introduce mixed plain-lint / tool-lint snapshot churn.

## Goals / Non-Goals

**Goals:** Add structured suggestions to lints where tigerstyle can express a
reviewable next edit, while keeping the applicability tier honest for each
pattern.

**Non-Goals:** Auto-fixing structural issues like `function_length` or
`assertion_density`. Those require human judgment about where to split
or what to assert.

## Decisions

### 1. Three tiers of applicability

**Choice:**
- `MachineApplicable`: `compound_assertion` for validated top-level
  conjunction rewrites (`assert!(a && b)`)
- `MaybeIncorrect`: declaration-only rename suggestions for `bool_naming` and
  `acronym_style`, plus `unchecked_division`, `raw_arithmetic_overflow`,
  `unchecked_narrowing`, and `platform_dependent_cast`
- `HasPlaceholders`: `unjustified_allow` and `numeric_units`

**Rationale:**
- `MachineApplicable` must preserve meaning for the validated pattern and keep
  the proof fixture compiling after application.
- Declaration-only renames do not rewrite use sites, so they cannot honestly
  claim auto-apply safety unless tigerstyle later grows full rename coverage for
  the validated scope.
- Guard and conversion suggestions may change control flow or error handling, so
  they stay review-required.
- Placeholder suggestions still need a maintainer-selected unit or real
  justification text.

### 2. Compound assertion autofix is limited to safe conjunction cases

**Choice:** Use `multipart_suggestion` only for validated top-level
`assert!(a && b)` cases. Top-level `assert!(a || b)` stays diagnostic-only or a
non-machine-applicable suggestion until the project proves a
semantics-preserving rewrite for that exact pattern.

**Rationale:** Replacing `assert!(a || b)` with two separate assertions would
strengthen the condition and therefore overclaim safety.

**Implementation:** Keep `compound_assertion` in its existing early-pass
`check_expr` path and recover the original macro call through
`expr.span.source_callsite()`. Parse only top-level conjunctions from the
callsite snippet, reject message-bearing forms like
`assert!(a && b, "...")` from the machine-applicable path, and split n-ary
conjunctions like `assert!(a && b && c)` into one assertion per top-level
clause. Emit one replacement for the full macro span plus one zero-width
insertion at the end of that span so the resulting `multipart_suggestion`
preserves the original line indentation and produces this exact shape:

```rust
assert!(a > 0);
assert!(b > 0);
```

For an indented three-clause case, the emitted replacement shape is:

```rust
        assert!(a > 0);
        assert!(b > 0);
        assert!(c > 0);
```

Message-bearing conjunctions still emit the ordinary lint warning but add no
structured suggestion. Top-level disjunctions also stay warning-only in this
change. This change keeps current `assert!`-only lint coverage; extending the
lint to `debug_assert!` would be separate work.

### 3. Rename suggestions stay declaration-scoped unless full propagation exists

**Choice:** `bool_naming` and `acronym_style` suggestions target the
identifier span at the declaration site unless implementation work also rewrites
all affected use sites needed to keep the validated fixture compiling.

**Rationale:** A declaration-only rename is still useful guidance, but it is a
review-required suggestion rather than an honest `MachineApplicable` autofix.
If later work adds complete same-scope propagation, the applicability can be
upgraded with matching proof.

### 4. Applicability is shown in a deterministic note and proven by two ignored rails

**Choice:** Every structured suggestion attaches `diag.note(...)` with the
exact strings:
- `structured suggestion applicability: MachineApplicable`
- `structured suggestion applicability: MaybeIncorrect`
- `structured suggestion applicability: HasPlaceholders`

The applicability note lands after the shared Tiger Style help text and before
`see Tiger Style guide: ...` in the human stderr output.

The concrete suggestion text contract for this rollout is:
- `bool_naming` -> `help: rename binding to \`is_active\`` style declaration-only rename
- `acronym_style` -> `help: rename \`RPCHandler\` to \`RpcHandler\`` style declaration-only rename
- `numeric_units` -> `help: rename with an explicit unit suffix like \`_ms\`, \`_secs\`, or \`_bytes\`` with placeholder replacement `timeout_<unit>`
- `unchecked_division` -> `help: guard \`count\` before division` with block-expression replacement `{ let __tigerstyle_divisor = count; assert!(__tigerstyle_divisor != 0); total / __tigerstyle_divisor }`
- `raw_arithmetic_overflow` -> `help: replace with saturating addition|subtraction|multiplication` using `(<lhs>).saturating_<op>(<rhs>)`
- `unchecked_narrowing` -> `help: replace with \`u32::try_from(...)\`` style conversion replacement
- `platform_dependent_cast` -> `help: replace with \`usize::try_from(...)\`` or `isize::try_from(...)` style conversion replacement
- `unjustified_allow` -> `help: add a local justification comment` with placeholder insertion `// TODO(tigerstyle): explain why this allow is safe here`

Proof splits into two ignored test rails:
- `cargo test --test ui ui -- --ignored --nocapture` refreshes and verifies the
  `.stderr` snapshots for every rollout lint plus the explicit guardrail
  fixtures
- `cargo test --test ui autofix_machine_applicable -- --ignored --nocapture`
  discovers `ui_tests/*.fixed`, creates a temporary crate with
  `Cargo.toml` containing `[package] name = "autofix_fixture"` and
  `edition = "2024"`, copies the corresponding fixture to `src/main.rs`,
  reuses the `tests/ui.rs`-built Dylint driver + lint library through
  `RUSTC_WORKSPACE_WRAPPER` and `DYLINT_LIBS`, runs `cargo fix`, compares the
  rewritten source to the checked-in `.fixed` file, then reruns
  `cargo check --quiet --bin autofix_fixture` and asserts that no warnings
  remain

**Rationale:** rustc's human diagnostics do not print applicability tiers on
their own, so the change needs a stable, reviewable note in `.stderr` output.
The second ignored rail pins the exact `.fixed` verification mechanism instead
of leaving review to infer whether a compile-pass or snapshot-only check was
used.

## Risks / Trade-offs

**[Incorrect auto-fixes]** → A `MachineApplicable` suggestion that produces
broken code would undermine trust. Mitigation: require `.fixed` proof for every
machine-applicable pattern plus one saved nix-devshell `cargo fix` transcript.

**[Applicability drift]** → Proposal, tasks, and implementation can easily
start overclaiming safety for rename or disjunction cases. Mitigation: keep the
spec tier contract explicit and add guardrail fixtures for those patterns.

**[Indentation breakage]** → Multi-line suggestions may not match local
indentation style. Mitigation: copy indentation from the original span's
leading whitespace.
