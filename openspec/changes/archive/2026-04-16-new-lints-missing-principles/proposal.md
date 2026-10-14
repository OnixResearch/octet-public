## Why

Several Tiger Style principles still have no lint coverage. The biggest gaps are
library defaults that silently change behavior, unbounded resource growth, and
API / naming patterns that push platform or readability problems downstream.

This change adds six lints that are concrete enough to verify with HIR/type
analysis and useful enough to validate against Aspen before any rollout-profile
promotion.

## What Changes

### Safety domain

- **`explicit_defaults`**: warn when code relies on the `Default` trait for a
  type defined outside the current crate, including sibling workspace crates.
  This covers `Default::default()`, `<T as Default>::default()`,
  `T::default()`, struct-update shorthand like `..Default::default()`, and
  user-written shorthand that expands to the same trait call such as
  `#[serde(default)]`.
- **`unbounded_channel`**: warn on known unbounded channel constructors from
  `std`, `crossbeam`, and Tokio, matched by resolved function identity rather
  than source spelling alone.
- **`unbounded_collection_growth`**: warn only on narrow, reviewable same-block
  cases where a local `Vec`/`HashMap`/`BTreeMap`/`VecDeque` binding is created
  empty and then grown in a loop without prior reservation or an explicit
  growth bound.

### Structure domain

- **`usize_in_public_api`**: warn when inherent `pub fn` signatures expose
  `usize` or `isize` at API boundaries.
- **`too_many_parameters`**: warn when a function exceeds a configurable
  parameter threshold (default 5), excluding `self` receivers.

### Naming domain

- **`negated_predicate`**: warn on boolean-returning functions and bool-typed
  local bindings whose names begin with whole-token negative prefixes such as
  `is_not_` or `has_no_`.

## Non-Goals

- Tool-lint namespace or group-attr migration; these lints follow the repo's
  existing registration model.
- Performance-domain lints.
- Cross-function or alias-heavy dataflow for `unbounded_collection_growth`.
  This change stays on same-block local bindings only.
- Automatic starter-profile promotion. New lints land first with downstream
  classification evidence, then a later change can promote low-noise lints.

## Capabilities

### New Capabilities
- `explicit_defaults`: detects trait-default construction on foreign-crate types
- `unbounded_channel`: detects known unbounded channel constructors
- `unbounded_collection_growth`: detects same-block unchecked collection growth in loops
- `usize_in_public_api`: detects platform-dependent integer types in inherent public APIs
- `too_many_parameters`: detects functions with excessive parameter counts
- `negated_predicate`: detects negative predicate naming on bool APIs and locals

## Default levels and rollout

- All six lints are intended to register **warn-by-default** in the repo.
- Downstream rollout starts as **inventory-first**, not starter-profile.
- Aspen evidence for this change lives under
  `openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/`, with one config
  snapshot plus starter-scope and `aspen-coordination` transcripts per lint,
  summarized by `validation-counts.txt`.
- A lint only becomes starter-profile eligible after Aspen evidence records its
  finding count and classifies it as low-noise. For this change,
  `starter-candidate` means <= 3 findings on the Aspen starter scope and <= 5
  findings on the focused `aspen-coordination` lib-only spotcheck; anything
  larger stays `inventory-first`.

## Impact

- **Files**: new lint modules in `src/safety/`, `src/structure/`, and
  `src/naming/`; registration in `src/lib.rs`; config update in `src/config.rs`
  for `max_parameters`; UI fixtures and snapshots per lint; rollout evidence in
  `openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/`; rollout guidance
  updates in `docs/downstream-rollout.md`, `templates/dylint.toml`, and
  `README.md`
- **APIs**: six new lint IDs, all warn-by-default
- **Dependencies**: none required up front; Tokio / crossbeam matching uses the
  existing repo approach of resolved def paths and fixture-local support crates
- **Testing**: per-lint UI fixtures, config fixture for `max_parameters`,
  explicit false-positive boundary fixtures, deterministic fixture inventories
  in `tests/fixtures/ui_fixture_set.json` and related snapshot metadata files,
  and Aspen evidence that records counts plus starter-vs-inventory-first
  classification for each lint
