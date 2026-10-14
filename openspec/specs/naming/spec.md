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

The lint MUST NOT fire on function parameters (only local bindings).

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
containing runs of 2 or more uppercase ASCII letters followed by a
lowercase letter (e.g., `RPCHandler`, `SQLQuery`, `TLSConfig`).

The lint MUST suggest the single-word capitalization (`RpcHandler`,
`SqlQuery`, `TlsConfig`).

The lint MUST NOT fire on all-uppercase names (`RPC`, `SQL`, `ID`).

The lint MUST NOT fire on two-letter prefixes that are standard Rust
(`Ok`, `Eq`, `Fn`).

The lint SHOULD allow a configurable allowlist for project-specific
exceptions.

#### Scenario: Multi-char uppercase acronym

- GIVEN `struct RPCHandler { ... }`
- WHEN the lint runs
- THEN it suggests renaming to `RpcHandler`

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
`size`, `length`, `capacity`, `limit`, `offset`, `port`) without a unit
suffix (`_ms`, `_secs`, `_us`, `_ns`, `_bytes`, `_kb`, `_mb`, `_gb`).

The lint MUST NOT fire on loop counters (`i`, `j`, `n`, `count`, `index`).

The lint MUST NOT fire on names that already end with a unit suffix.

#### Scenario: Bare timeout name

- GIVEN `let timeout = Duration::from_secs(5);`
- WHEN the lint runs
- THEN it suggests `timeout_secs` or `timeout_ms`

#### Scenario: Already suffixed

- GIVEN `let timeout_ms: u64 = 5000;`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Floating Point for Currency

The lint MUST be able to warn on struct fields or function return types using
`f32` or `f64` whose name contains `cost`, `price`, `budget`,
`balance`, `amount`, `fee`, or `total_cost`.

The lint MUST suggest using integer types with explicit units
(e.g., `cost_microdollars: u64` or a newtype wrapper).

The lint MUST NOT fire on fields named `percentage`, `ratio`, or
`factor` (those are legitimately floating point).

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

### Requirement: Negated Predicate Detection

The lint MUST warn on:
- boolean-returning functions
- bool-typed local bindings

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

### Requirement: Non-Trait Import Detection [r[non-trait-import-detection]]

The system MUST provide a warn-by-default naming/import lint named `non_trait_imports`.

The lint MUST warn on `use` imports that bring non-trait items directly into local scope.

The lint MUST allow trait imports, including trait imports renamed to `_` for method resolution.

The lint MUST allow public owner-type re-exports when the imported item is intentionally part of the module's public API.

The lint MUST NOT warn on prelude imports or generated code.

#### Scenario: Struct imported into local scope [r[non-trait-import-detection.scenario.struct-imported]]

- GIVEN production code with `use crate::addr::Addr;` followed by local uses of `Addr`
- WHEN the lint runs
- THEN `non_trait_imports` warns to prefer the qualified owner path at the use site

#### Scenario: Trait import for method resolution [r[non-trait-import-detection.scenario.trait-import]]

- GIVEN production code with `use std::io::Write as _;`
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

The lint MUST NOT warn when the repeated word is required to disambiguate a public compatibility name or an externally mandated trait/API name.

#### Scenario: Function repeats module name [r[path-segment-repetition-detection.scenario.function-repeats-module]]

- GIVEN production code in module `guest` that defines `fn spawn_guest_vm(...)`
- WHEN the lint runs
- THEN `path_segment_repetition` warns that `guest` is already supplied by the module path

#### Scenario: Non-repeating item name [r[path-segment-repetition-detection.scenario.non-repeating-item]]

- GIVEN production code in module `guest` that defines `fn spawn_vm(...)`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire

#### Scenario: Compatibility name repeats path [r[path-segment-repetition-detection.scenario.compatibility-name]]

- GIVEN production code in module `slotcar` that defines a documented compatibility item named `slotcar_config`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire

