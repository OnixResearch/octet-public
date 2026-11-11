# Naming Specification

## Purpose

Tiger Style naming requirements for lint enforcement and agent-facing skill
coverage: boolean prefixes, acronym capitalization, unit suffixes, and
names that make obligations explicit.
## Requirements
### Requirement: Behavioral Naming Coverage In Skill

The Tiger Style skill MUST include the omitted naming guidance from the
reference.

The skill MUST cover:

- behavioral naming that signals ownership or lifecycle obligations
- avoiding name overloading across unrelated concepts
- using option/config structs when multiple parameters share a type
- documenting the "why" rather than restating the "what"
- proper comment style as complete sentences

#### Scenario: Ambiguous same-type parameters

- GIVEN a function with multiple `u64` parameters
- WHEN an agent follows the skill
- THEN it is prompted to prefer a named options struct over positional ambiguity

#### Scenario: Ownership-signaling names

- GIVEN a parameter that transfers ownership
- WHEN an agent names it under Tiger Style
- THEN the name can signal the ownership contract, such as `owned_buffer`

### Requirement: Boolean Naming Prefix

The lint MUST detect `let` bindings of type `bool` (inferred or annotated)
whose name does not start with `is_`, `has_`, `should_`, `can_`, `was_`,
`will_`, or `needs_`.

The lint MUST suggest renaming with the appropriate prefix.

The lint MUST NOT fire on destructuring bindings (`let Foo { active } = ...`).

The lint MUST NOT fire on function parameters (only local bindings), including
`async fn` and async closure parameters, which rustc lowers into `let`
statements inside the async body.

The lint MAY fire on `mut` boolean bindings.

#### Scenario: Bare boolean name

- GIVEN `let active = connections > 0;`
- WHEN the lint runs
- THEN it suggests renaming to `is_active`

#### Scenario: Already prefixed

- GIVEN `let is_active = connections > 0;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-boolean variable with boolean-sounding name

- GIVEN `let enabled: u32 = config.flags;`
- WHEN the lint runs
- THEN it does NOT fire (type is not `bool`)

### Requirement: Acronym Capitalization Style

The lint MUST detect type names, enum variant names, and trait names
containing an acronym: a run of 2 or more uppercase ASCII letters that is
followed by a capitalized word (e.g., `RPCHandler`, `SQLQuery`,
`TLSConfig`), followed by a digit (`HTTP2Server`), or ends a mixed-case name
(`ParseJSON`, `UserID`). Every such run in a name MUST be checked.

The lint MUST suggest the single-word capitalization (`RpcHandler`,
`SqlQuery`, `TlsConfig`, `Http2Server`, `ParseJson`, `UserId`).

The lint MUST NOT fire on all-uppercase names (`RPC`, `SQL`, `ID`).

The lint MUST NOT fire on two-letter prefixes that are standard Rust
(`Ok`, `Eq`, `Fn`).

The lint SHOULD allow a configurable allowlist for project-specific
exceptions.

#### Scenario: Multi-char uppercase acronym

- GIVEN `struct RPCHandler { ... }`
- WHEN the lint runs
- THEN it suggests renaming to `RpcHandler`

#### Scenario: Trailing and digit-adjacent acronyms

- GIVEN `struct ParseJSON;` and `struct HTTP2Server;`
- WHEN the lint runs
- THEN it suggests renaming to `ParseJson` and `Http2Server`

#### Scenario: Trailing uppercase (plural)

- GIVEN a field or variable named `node_IDs`
- WHEN the lint runs
- THEN it suggests `node_ids` (this is a naming convention, not a type lint)

#### Scenario: All-caps constant

- GIVEN `const MAX_RPC_SIZE: usize = 1024;`
- WHEN the lint runs
- THEN it does NOT fire (constants are `SCREAMING_SNAKE_CASE`)

### Requirement: Numeric Variable Unit Suffix

The lint MUST be able to detect numeric variable bindings whose name contains
a time or size word (`timeout`, `delay`, `duration`, `interval`, `latency`,
`size`, `length`, `capacity`, `limit`, `offset`) without a unit suffix.
Quantity words MUST match as whole identifier words, split at `_` and
camelCase humps, so `resized_total` does not count as `size`.

Unit suffixes are `_ns`, `_nanos`, `_us`, `_micros`, `_ms`, `_millis`,
`_sec`, `_secs`, `_seconds`, `_mins`, `_minutes`, `_hours`, `_days`, `_bits`,
`_bytes`, `_kb`, `_mb`, `_gb`, `_kib`, `_mib`, and `_gib`.

The lint MUST NOT fire on loop counters and lengths (`i`, `j`, `k`, `n`,
`count`, `index`, `idx`, `len`).

The lint MUST NOT fire on names that already end with a unit suffix,
including spelled-out time units (`timeout_seconds`) and binary size units
(`buffer_size_kib`).

The `let` statements rustc generates for `async fn` and async closure
parameters MUST NOT count as local bindings; an `async fn` parameter is
checked once, as a parameter.

#### Scenario: Bare timeout name

- GIVEN `let timeout = Duration::from_secs(5);`
- WHEN the lint runs
- THEN it suggests `timeout_secs` or `timeout_ms`

#### Scenario: Already suffixed

- GIVEN `let timeout_ms: u64 = 5000;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Spelled-out or binary unit suffix

- GIVEN `let timeout_seconds: u64 = 5;` or `let buffer_size_kib: usize = 4;`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Floating Point for Currency

The lint MUST be able to warn on struct fields or function return types using
`f32` or `f64` whose name contains one of the whole words `cost`, `price`,
`budget`, `balance`, `amount`, or `fee`. Words are split at `_` and camelCase
humps and may carry a plural `s` or `es`, so `total_cost` and `monthly_fees`
warn, while a currency word inside another word does not: `load_balancer_weight`,
`feedback_score`, and `priceless_rank` stay quiet. For return types, the name is
the function or method name; free functions, inherent methods, and trait
method declarations are checked, while trait implementations inherit the
trait's signature and are not reported again.

The lint MUST suggest using integer types with explicit units
(e.g., `cost_microdollars: u64` or a newtype wrapper).

The lint MUST NOT fire on names with a whole `_`-separated `percentage`,
`ratio`, `factor`, or `rate` token (those are legitimately floating point).
Skip words MUST NOT match inside other tokens, so `registration_fee`,
`operation_cost`, and `corporate_budget` still warn.

This lint exists because clankers' `CostTracker` uses `f64` for all
cost calculations (`cost_usd: f64`, `total_cost: f64`), while
verified-logic's `rate_limit.rs` specifically uses integer
"millitokens" to avoid floating point precision issues. The IEEE 754
representation loses precision above 2^53 — $9,007,199,254,740,992
sounds like a lot, but accumulated rounding across thousands of API
calls adds up.

#### Scenario: Float cost field

- GIVEN `pub struct Usage { pub cost_usd: f64 }`
- WHEN the lint runs
- THEN it warns: "floating point type for currency — consider integer microdollars"

#### Scenario: Float percentage field

- GIVEN `pub struct Breakdown { pub percentage: f32 }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Skip word inside another token

- GIVEN `pub struct Charges { pub registration_fee: f64 }`
- WHEN the lint runs
- THEN it warns, because `ratio` inside `registration` is not a whole token

#### Scenario: Float return type of a currency function

- GIVEN `fn total_cost() -> f64`
- WHEN the lint runs
- THEN it warns on the return type

### Requirement: Negated Predicate Detection

The lint MUST warn on:
- boolean-returning functions
- bool-typed local bindings (user-written `let` bindings, not the `let`
  statements rustc generates for `async fn` and async closure parameters)

when their names begin with one of these whole-token negative prefixes:
- `is_not_`
- `has_no_`
- `has_not_`
- `lacks_`
- `cannot_`
- `cant_`
- `does_not_`
- `doesnt_`
- `isnt_`
- `wasnt_`
- `wont_`
- `no_`

The lint MUST suggest a positive reformulation.

The lint MUST use word-boundary semantics for prefix matching, so near-miss
names like `notify` or `notion_ready` do NOT count as negative prefixes.

The `no_` prefix is intentionally broad for bool names. A bool helper like
`no_capacity` still counts as a negated predicate; only infix or near-miss
spellings stay quiet.

The lint MUST NOT fire on non-boolean bindings or functions.

The lint MUST NOT fire on a private helper function named exactly `not`.

A machine-applicable rename of a bool local that negates its value MUST keep
every read inside a function-like macro (`name!(...)`) valid. A `!name` the
user wrote collapses to the positive name, so `assert!(!no_items)` becomes
`assert!(has_items)`. A read that is a whole macro argument keeps its source
spans even as a `$x:tt`, where `!name` is two tokens and can stop matching
the arm, so it MUST be written `(!name)` when the macro uses it as an operand
of `!` or a binary operator. In any other macro-generated position, such as an
`if` condition, the rename MUST fall back to a review-required suggestion,
because rustc's `unused_parens` lint flags `(!name)` there and its own fix
removes the parentheses. Attribute macros do not count as macro arguments.
A read passed as an `$x:ident` or `$x:path` argument MUST fall back to the
review-required rename, because neither `!name` nor `(!name)` is an identifier
or a path. rustc gives these arguments a span from the macro expansion, which
the rename already treats as macro-generated.

#### Scenario: Negated function name

- GIVEN `fn is_not_ready(state: &State) -> bool`
- WHEN the lint runs
- THEN it warns to prefer a positive form like `is_ready`

#### Scenario: Positive function name

- GIVEN `fn is_ready(state: &State) -> bool`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Near-miss identifier

- GIVEN `fn notify(state: &State) -> bool`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-boolean variable

- GIVEN `let no_progress: usize = retries_left;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Read passed to a token-tree macro argument

- GIVEN `let has_no_items = len == 0;` read as `ensure!(has_no_items)` where `ensure!` takes `$cond:tt` and expands to `if !$cond { ... }`
- WHEN the lint runs
- THEN it offers a machine-applicable rewrite to `ensure!((!has_items))`

#### Scenario: Token-tree macro argument used as a condition

- GIVEN `let no_items = len == 0;` read as `bail_if!(no_items)` where `bail_if!` takes `$cond:tt` and expands to `if $cond { ... }`
- WHEN the lint runs
- THEN it warns with a review-required rename instead of a machine-applicable rewrite

#### Scenario: Identifier or path macro argument

- GIVEN `let no_values = values.is_empty();` read as `require!(no_values)` where `require!` takes `$flag:ident` or `$flag:path`
- WHEN the lint runs
- THEN it warns with a review-required rename instead of a machine-applicable `require!((!has_values))` rewrite

### Requirement: Underscore Module Filename Detection [r[underscore-module-filename-detection]]

The system MUST provide a warn-by-default naming lint named `underscore_in_module_filename`.

The lint MUST warn when a production module file name contains an underscore, such as `compute_vm.rs`.

The lint MUST suggest replacing compound file names with a directory hierarchy when practical, such as `compute/vm.rs`.

The lint MUST NOT warn on crate roots, generated files, fixture files, or files outside the linted package root.

#### Scenario: Compound module filename [r[underscore-module-filename-detection.scenario.compound-module-filename]]

- GIVEN a production module file at `src/compute_vm.rs`
- WHEN the lint runs
- THEN `underscore_in_module_filename` warns and suggests a directory-based module layout

#### Scenario: Directory-based module filename [r[underscore-module-filename-detection.scenario.directory-based-module-filename]]

- GIVEN production module files at `src/compute/vm.rs` and `src/compute/mod.rs`
- WHEN the lint runs
- THEN `underscore_in_module_filename` does NOT fire

#### Scenario: Underscore crate root [r[underscore-module-filename-detection.scenario.underscore-crate-root]]

- GIVEN a binary or example crate root such as `src/bin/load_gen.rs` or `examples/basic_usage.rs`
- WHEN the lint runs
- THEN `underscore_in_module_filename` does NOT fire for the crate root file, because its name is the Cargo target name
- AND underscore module files that the crate root declares still warn

### Requirement: Non-Trait Import Detection [r[non-trait-import-detection]]

The system MUST provide a warn-by-default naming/import lint named `non_trait_imports`.

The lint MUST warn on `use` imports that bring non-trait items directly into local scope.

The lint MUST allow trait imports, including trait imports renamed to `_` for method resolution and trait imports that also bring in a same-named derive macro, such as `use std::hash::Hash;`.

The lint MUST allow public owner-type re-exports when the imported item is intentionally part of the module's public API.

The lint MUST NOT warn on prelude imports or generated code.

The lint MUST NOT exempt a file because of a substring of its source path: a downstream file such as `crates/gui_tests/src/lib.rs` or one under `tests/fixtures/` is checked like any other source.

#### Scenario: Struct imported into local scope [r[non-trait-import-detection.scenario.struct-imported]]

- GIVEN production code with `use crate::addr::Addr;` followed by local uses of `Addr`
- WHEN the lint runs
- THEN `non_trait_imports` warns to prefer the qualified owner path at the use site

#### Scenario: Trait import for method resolution [r[non-trait-import-detection.scenario.trait-import]]

- GIVEN production code with `use std::io::Write as _;` or `use std::hash::Hash;` (a trait plus its same-named derive macro)
- WHEN the lint runs
- THEN `non_trait_imports` does NOT fire

#### Scenario: Public owner-type re-export [r[non-trait-import-detection.scenario.public-owner-reexport]]

- GIVEN production code with `pub use crate::addr::Addr;` that is documented as part of the module public API
- WHEN the lint runs
- THEN `non_trait_imports` does NOT fire

### Requirement: Renamed Import Detection [r[renamed-import-detection]]

The system MUST provide a warn-by-default naming/import lint named `renamed_imports`.

The lint MUST warn on `use` statements that rename an import with `as`.

The lint MUST NOT warn on `as _` trait imports used only for method resolution.

The lint MUST NOT warn when the imported item is a public re-export that preserves a documented compatibility alias.

The lint MUST treat an alias as documented when its doc comments, or the contiguous comment lines between and directly above its outer attributes, contain `compat` or `compatibility` as a whole word. Words inside other words, such as `compat` in `incompatible`, do not count. For an alias inside a `{...}` group, such as `pub use crate::octet::{Config as LegacyConfig};`, the comment lines above the whole `use` statement count, on one line or across several.

The lint MUST NOT exempt a file because of a substring of its source path: a downstream file such as `crates/gui_tests/src/lib.rs` or one under `tests/fixtures/` is checked like any other source.

#### Scenario: Renamed private import [r[renamed-import-detection.scenario.renamed-private-import]]

- GIVEN production code with `use crate::transport::Config as TransportConfig;`
- WHEN the lint runs
- THEN `renamed_imports` warns that the rename hides the original searchable name

#### Scenario: Trait import as underscore [r[renamed-import-detection.scenario.trait-import-underscore]]

- GIVEN production code with `use std::fmt::Write as _;`
- WHEN the lint runs
- THEN `renamed_imports` does NOT fire

#### Scenario: Documented compatibility alias [r[renamed-import-detection.scenario.compatibility-alias]]

- GIVEN production code with `pub use crate::octet::Config as SlotcarConfig;` and adjacent compatibility documentation
- WHEN the lint runs
- THEN `renamed_imports` does NOT fire

### Requirement: Path Segment Repetition Detection [r[path-segment-repetition-detection]]

The system MUST provide a warn-by-default naming lint named `path_segment_repetition`.

The lint MUST warn when an item name repeats underscore-separated words that are already present in the crate name or ancestor module path.

The lint MUST compare normalized snake-case word segments, treating hyphens in crate names as separators.

The crate name MUST count only for library crates and the `--test` build of a library, because only a library's name appears in paths users write. Crate names that Cargo derives from target or file names MUST NOT count: build scripts (`build_script_build`), binaries, examples, integration tests, and benches.

The lint MUST NOT warn when the repeated word is required to disambiguate a public compatibility name or an externally mandated trait/API name.

The lint MUST treat an item as such a name when its doc comments, or the contiguous comment lines between and directly above its outer attributes, contain `compat`, `compatibility`, `external`, `mandated`, or `api` as a whole word. Words inside other words, such as `api` in `rapid`, do not count.

The lint MUST NOT exempt a file because of a substring of its source path: a downstream file such as `crates/gui_tests/src/lib.rs` or one under `tests/fixtures/` is checked like any other source.

#### Scenario: Function repeats module name [r[path-segment-repetition-detection.scenario.function-repeats-module]]

- GIVEN production code in module `guest` that defines `fn spawn_guest_vm(...)`
- WHEN the lint runs
- THEN `path_segment_repetition` warns that `guest` is already supplied by the module path

#### Scenario: Item repeats crate name [r[path-segment-repetition-detection.scenario.item-repeats-crate]]

- GIVEN production code in library crate `octet-vm` that defines `fn vm_config()` at the crate root or in any module
- WHEN the lint runs
- THEN `path_segment_repetition` warns that `vm` is already supplied by the crate name

#### Scenario: Non-library crate name [r[path-segment-repetition-detection.scenario.non-library-crate]]

- GIVEN a build script that defines `fn build_protos()`, or a binary `src/bin/load_gen.rs` that defines `fn load_tool()`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire, because `build_script_build` and `load_gen` are not path segments

#### Scenario: Non-repeating item name [r[path-segment-repetition-detection.scenario.non-repeating-item]]

- GIVEN production code in module `guest` that defines `fn spawn_vm(...)`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire

#### Scenario: Compatibility name repeats path [r[path-segment-repetition-detection.scenario.compatibility-name]]

- GIVEN production code in module `slotcar` that defines a documented compatibility item named `slotcar_config`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire

