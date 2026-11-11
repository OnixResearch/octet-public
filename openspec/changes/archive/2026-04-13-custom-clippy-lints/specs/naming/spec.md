# Naming Lints — Delta Spec

## ADDED Requirements

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
