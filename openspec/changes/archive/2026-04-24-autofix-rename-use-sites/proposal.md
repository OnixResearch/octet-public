## Why

The autofix spec's applicability contract explicitly blocks rename lints from
`MachineApplicable` because they only rewrite the declaration span today.
Consumers see a useful suggestion but cannot run `cargo fix` to apply it --
they must manually rename every use site. For local variable renames
(`bool_naming`, `negated_predicate` on let bindings) the use sites are all
within the same function body, making automated collection both safe and
straightforward.

The wave-1 autofix rollout proved the suggestion infrastructure, `.fixed` proof
rail, and `cargo fix` transcript pipeline. This change extends that pipeline to
cover use-site rewriting for the simplest rename patterns, promoting them from
`MaybeIncorrect` to `MachineApplicable` with full proof artifacts.

## What Changes

- **Use-site collection for local variable renames**: `bool_naming` and
  `negated_predicate` (when firing on `check_local`) collect all references to
  the flagged binding within the enclosing function body and emit multipart
  suggestions covering declaration + all use sites.

- **MachineApplicable promotion**: local-variable patterns in `bool_naming` and
  `negated_predicate` upgrade from `MaybeIncorrect` / `HasPlaceholders` to
  `MachineApplicable` when use-site collection succeeds and all spans are
  rewritable.

- **Autofix spec delta**: the applicability contract gains a new clause
  permitting `MachineApplicable` for rename lints that also rewrite use sites.

- **Proof artifacts**: `.fixed` fixtures and a `cargo fix` transcript for each
  promoted pattern.

## Out of Scope

- **`acronym_style`**: fires on item-level names (structs, enums, functions,
  fields, variants) whose use sites span the entire crate. Cross-crate
  references cannot be rewritten. Stays `MaybeIncorrect` until a separate change
  addresses item-level use-site collection.

- **`negated_predicate` on functions/methods**: function and method renames
  (`check_item`, `check_impl_item`, `check_trait_item`) have crate-wide or
  cross-crate scope. Public API renames break downstream consumers. These stay
  at their current applicability tier.

- **Cross-crate use sites**: only within-crate references are rewritable by a
  Dylint lint. No attempt to rewrite external consumers.

- **Macro-generated bindings**: if the declaration or any use site has a span
  from macro expansion, the suggestion falls back to `MaybeIncorrect` rather
  than claiming `MachineApplicable`.

## Impact

- **Files**: `src/naming/bool_naming.rs`, `src/naming/negated_predicate.rs`,
  possibly a shared `src/naming/use_site_collector.rs` helper
- **UI fixtures**: new `.fixed` files for `bool_naming` and `negated_predicate`
  local-variable patterns, updated `.stderr` snapshots
- **Spec**: delta to `openspec/specs/autofix/spec.md`
- **Dependencies**: no new crate dependencies; uses existing `rustc_hir`
  traversal
- **Testing**: existing `tests/ui.rs` ignored rails for snapshot refresh and
  `autofix_machine_applicable` verification
