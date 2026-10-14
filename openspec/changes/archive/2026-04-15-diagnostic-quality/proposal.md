## Why

Phase 1 and Phase 2 of `diagnostic-quality` already centralized diagnostic
metadata and help/note emission, but Phase 3 exposed a deeper namespace
problem. Tigerstyle currently declares plain `declare_lint!` statics and feeds
bare lint names through `sess.opts.lint_opts`. That works for bare config keys,
but it does not make the documented tool-scoped API real: tool-scoped domain
attrs like `#![allow(tigerstyle::safety)]` only resolve against true tool
lints.

A naive switch to `declare_tool_lint!` fixes the tool-scoped group problem but
breaks the current bare-name config path with `unknown lint tool: tigerstyle`.
This change needs one coherent model: true tool lints for the public
`tigerstyle::...` namespace, plus a compatibility path that keeps existing
`dylint.toml` bare keys and repo-local bare fixtures working.

## What Changes

- Keep the shared diagnostic wrapper, metadata inventory, and canonical
  `Tiger Style (...)` / `see Tiger Style guide: ...` wording added in the
  foundation phases
- Gate the full namespace migration behind a single-lint spike (use
  `no_unwrap`) that proves `declare_tool_lint!` and one-member bare
  compatibility aliases work under this repo's Dylint harness
- Convert the 25 existing consumer-facing lint statics to true tool lints
  under the `tigerstyle::...` namespace so per-lint attrs and domain groups
  resolve through the same rustc model
- Add one companion safety lint, `unjustified_no_panic_allow`, so the
  `#[allow(tigerstyle::no_panic)]` escape hatch stays reviewable after the
  tool-lint migration
- Register domain groups that match spec domains (`tigerstyle::safety`,
  `tigerstyle::naming`, `tigerstyle::structure`, `tigerstyle::purity`,
  `tigerstyle::concurrency`) from the same shared metadata inventory
- Preserve bare `dylint.toml` keys (`disabled_lints`, `lint_levels`) and
  existing bare per-lint fixture paths by adding compatibility aliases derived
  from the same inventory instead of keeping fake plain-lint registrations
- Update README, downstream rollout docs, templates, snapshots, and UI
  fixtures so canonical examples use `tigerstyle::...` names while standalone
  raw-rustc tool-scoped fixtures document the nightly `register_tool`
  requirement explicitly

Example improved diagnostic for `ambient_clock`:

```
warning: direct wall-clock read in production code
  --> src/coordination/timeout.rs:42:18
   |
42 |     let now = Instant::now();
   |               ^^^^^^^^^^^^^^
   |
   = help: Tiger Style (Functional Core): inject time as a parameter for core logic, or isolate the clock read in a shell-boundary helper with a narrow allow reason. Example: `#[allow(tigerstyle::ambient_clock, reason = "shell boundary clock read")] fn read_now() -> Instant { Instant::now() }`
   = note: see Tiger Style guide: Functional Core / Imperative Shell
```

Example improved diagnostic for `compound_assertion`:

```
warning: compound assertion should be split into separate assertions
  --> src/verified/lock.rs:15:5
   |
15 |     assert!(a > 0 && b > 0);
   |     ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: Tiger Style (Assertion Quality): split compound assertions so the failing clause is obvious. Example: `assert!(a > 0); assert!(b > 0);`
   = note: see Tiger Style guide: Assertion Density
```

## Capabilities

### New Capabilities
- `rich-diagnostics`: Every lint emits help text with a principle name,
  approved pattern, and guide-section pointer
- `native-tool-lint-namespace`: Every documented tigerstyle lint and domain
  group resolves through a true `tigerstyle::...` tool-lint identity
- `lint-groups`: Domain-level lint groups for bulk `#[allow]` / `#[deny]`

### Modified Capabilities
- `per-lint-configuration`: bare `dylint.toml` keys keep working even though
  the canonical lint namespace becomes `tigerstyle::...`
- All 26 lint diagnostics get richer output

## Impact

- **Files**: Every lint source file, `src/diagnostics.rs`, `src/lib.rs`,
  config helpers, README/docs/templates, UI fixtures, and inventory snapshots
- **APIs**: Canonical tool-lint IDs and tool-scoped domain-group IDs become
  the documented source-attr API; bare config keys remain supported; the
  `no_panic` escape hatch is now audited by a dedicated companion lint
- **Dependencies**: None required for the design itself
- **Testing**: Update snapshots, add canonical tool-group fixtures, preserve
  bare config compatibility fixtures, prove the single-lint spike before the
  bulk migration, revalidate the archived Aspen starter-rollout reference at
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`, and run
  `cargo test --lib`, `cargo test --test ui -- --ignored --nocapture`, and
  `nix flake check`
