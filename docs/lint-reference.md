# Lint reference

This page lists the shipped Octet lints, their default levels, and the
supported suppression and config paths.

For consumer setup, see `consumer-guide.md`.
For checked-contract ladder context, see `trust-boundary-evidence.md#checked-contract-ladder`.
For the landing-page naming map, see `../README.md`.

## Catalog summary

`octet` ships 77 lints across 6 domains.

| Domain | Group | Count |
| --- | --- | ---: |
| safety | `tigerstyle::safety` | 27 |
| naming | `tigerstyle::naming` | 9 |
| performance | `tigerstyle::performance` | 2 |
| structure | `tigerstyle::structure` | 21 |
| purity | `tigerstyle::purity` | 16 |
| concurrency | `tigerstyle::concurrency` | 2 |

## Suppression and level controls

Source attrs and group attrs use the current live tool namespace:

```rust
#[allow(tigerstyle::unchecked_narrowing)]
#![deny(tigerstyle::naming)]
#![warn(tigerstyle::numeric_units)]
```

Keep config keys bare in `dylint.toml`:

```toml
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
numeric_units = "warn"
no_unwrap = "deny"
```

Rules to keep straight:

- source attrs and domain groups currently use `tigerstyle::...` until the namespace migration lands
- `disabled_lints` and `[octet.lint_levels]` use bare lint names
- `disabled_lints` wins over `lint_levels`
- explicit command-line levels such as `-D numeric_units` in `DYLINT_RUSTFLAGS` win over `lint_levels`; a selected strict profile's required denies win over both, and a profiled compilation fails with an error when `--force-warn` or `--cap-lints` below `deny` would weaken one of them
- `[octet]` replaces the legacy `[slotcar]` and `[tigerstyle]` sections as a whole, even when it holds only `[octet.lint_levels]`; a shadowed legacy section is not parsed and the run warns that it is ignored
- allow-by-default lints can be turned on with `#![warn(...)]` or a lint-level override
- standalone raw-rustc crates and UI fixtures that use tool-scoped attrs need:

```rust
#![feature(register_tool)]
#![register_tool(tigerstyle)]
```

## Config keys

| Key | Default | Used by | Notes |
| --- | --- | --- | --- |
| `function_length_max` | `70` | `function_length` | maximum non-blank, non-comment lines between a function body's braces; `#[test]` functions are exempt |
| `excessive_file_length_max` | `300` | `excessive_file_length` | maximum physical lines in a production source file; target, generated, fixture, `.cargo`, and vendored paths are excluded by their path components below the workspace root |
| `module_file_count_max` | `7` | `module_file_count` | maximum Rust source files in one module directory; target, generated, `.cargo`, and vendored paths are excluded by their path components below the workspace root |
| `nesting_max_depth` | `3` | `nested_conditionals` | maximum `if` / `match` nesting depth |
| `assertion_density_min_lines` | `20` | `assertion_density` | minimum function size before assertion-density checks apply |
| `assertion_density_min_assertions` | `2` | `assertion_density` | minimum assertion count once the line threshold is crossed |
| `max_parameters` | `5` | `too_many_parameters` | maximum counted parameters before warning |
| `trait_method_max` | `7` | `fat_trait` | maximum trait methods before warning |
| `impl_blocks_max` | `4` | `type_accumulation` | maximum inherent impl blocks per resolved self type before advisory |
| `module_pub_items_max` | `12` | `type_accumulation` | maximum public items per module, including the crate root, before advisory; the public `main` and test registrations that the `--test` harness generates are not counted |
| `core_source_scopes` | `[]` | `concrete_construction_in_core`, `global_state_in_core`, `impure_call_in_core`, `interior_mutability_in_core`, `logging_in_core`, `mutating_input_in_pure`, `resource_return_from_core`, `spawn_in_core` | workspace-relative path prefixes (directories or files) where core-boundary checks run; entries are trimmed, a leading `./` and trailing `/` are dropped, and matching is by whole path components from the workspace root, so `crates/core` does not cover `crates/core-shell` and `domain-core/src` does not cover `crates/domain-core/src`; each entry always matches files rustc compiles through its own path, and an entry that resolves on disk, through symlinks or `..`, to another path inside the workspace also covers that canonical path, so `linked -> src` covers `src/`; an entry that normalizes to empty warns and is ignored; an entry that names no existing file or directory or resolves to the workspace root itself warns because it matches nothing; an entry whose target lies outside the workspace, such as `src/core -> ../../shared/core`, warns that it matches only files compiled through its in-root path |
| `core_value_provider_types` | `[]` | `concrete_construction_in_core` | exact compiler definition paths of project-reviewed pure value ADTs, such as `reviewed_values::Value`; aliases resolve to the same definition path, while crate-only names, prefixes, suffixes, and unselected siblings do not match |
| `durability_profile_path` | omitted | `discarded_sync_result`, `flush_only_durability`, `retry_sync_after_error` | workspace-relative path to a generated durability-claims profile JSON; the profile declares analyzed source scopes, boundary functions, required observation classes, uncertain-outcome types, configured commit surfaces, and adapter observations; an unreadable or invalid profile is reported as an error and the durability lints stay inert |
| `nominal_domain_mode` | omitted | `primitive_domain_alias`, `raw_domain_value`, `newtype_invariant_bypass` | `inventory` warns for exact and heuristic findings and caps `lint_levels` denies for these lints at `warn` (with a config warning), `warn` reports exact findings only, and `deny` denies exact findings only; heuristic candidates are skipped wherever the effective level is above `warn`, such as under a command-line `-D primitive_domain_alias`; a selected `bounded-systems` or `deterministic-core` strict profile forces `deny`, so no cap applies or is reported |
| `sealed_enum_marker` | `"octet::sealed_enum"` | `fragile_exhaustive_enum_match` | attribute marker that suppresses fragile local exhaustive enum-match diagnostics |
| `state_machine_core_marker` | `"octet::state_machine_core"` | `state_machine_exhaustiveness` | attribute marker that opts transition functions into explicit state/event matrix checks |
| `state_machine_enum_marker` | `"octet::state_machine_enum"` | `fragile_exhaustive_enum_match`, `state_machine_exhaustiveness` | required marker for deliberately closed FSM state/event enums; missing markers fail closed in FSM transition cores |
| `state_machine_core_scopes` | `[]` | `state_machine_exhaustiveness` | workspace-relative path prefixes, normalized, resolved, and matched like `core_source_scopes`, that opt transition functions into the FSM exhaustive-default profile |
| `state_machine_generated_adapter_scopes` | `[]` | `state_machine_exhaustiveness` | generated adapter/source path prefixes, normalized, resolved, and matched like `core_source_scopes`, excluded from the pure transition-core FSM profile |
| `indexable_lint_exceptions` | `["rustc_private_aliases", "lint_identity_names", "generated_searchable_names"]` | naming/indexable lints | named exception classes for rustc-private aliases, lint-identity-preserving names, and generated source; `generated_searchable_names` exempts `non_trait_imports`, `renamed_imports`, and `path_segment_repetition` findings in source files outside the workspace root or with a whole `target`, `generated`, `OUT_DIR`, `rustlib`, `.cargo`, or `*vendor-cargo-deps*` path component below it, never because of `use` path segments, module names, or path substrings |
| `acronym_allowlist` | `[]` | `acronym_style` | case-sensitive allowlist such as `IO` or `ID` |
| `disabled_lints` | `[]` | all lints | bare lint names only; disabled lints do not register |
| `[octet.lint_levels]` | `{}` | all lints | bare lint names mapped to `allow`, `warn`, or `deny` |

Unknown config keys (such as a misspelled `[octet.lint-levels]` table or `disabled_lint` key), unknown lint names, and invalid level values warn and are ignored.

## Safety lints (`tigerstyle::safety`)

| Lint | Default | Summary |
| --- | --- | --- |
| `compound_assertion` | `warn` | split `assert!(a && b)` into separate assertions |
| `ignored_result` | `warn` | avoid silent `let _ = expr` drops for fallible results |
| `drop_must_use` | `warn` | handle must-use values explicitly instead of hiding them behind `drop(...)` |
| `fallible_int_fallback` | `warn` | propagate integer conversion failures instead of replacing them with sentinels or defaults |
| `no_unwrap` | `warn` | ban `.unwrap()` and `.expect()` in production code |
| `no_panic` | `warn` | ban `panic!()`, `unimplemented!()`, and `unreachable!()` in production code |
| `no_todo` | `warn` | ban `todo!()` in production code |
| `unjustified_no_todo_allow` | `warn` | require a local `//` justification for `allow(no_todo)` / `allow(tigerstyle::no_todo)` |
| `no_recursion` | `warn` | flag direct self-recursion |
| `unchecked_narrowing` | `warn` | flag `as u32`-style narrowing without a checked bound |
| `unchecked_division` | `warn` | require a zero guard before dividing by a variable |
| `unbounded_loop` | `warn` | flag `loop {}` without an explicit bound or exit condition |
| `catch_all_on_enum` | `warn` | flag `_` match arms that hide future enum variants |
| `foreign_enum_catch_all` | `allow` | flag `_` match arms over enums defined in another crate |
| `fragile_exhaustive_enum_match` | `allow` | flag exhaustive matches over same-crate enums without a fallback |
| `state_machine_exhaustiveness` | `warn` | check marked state-machine transition cores for explicit state/event handling |
| `explicit_defaults` | `warn` | avoid foreign `Default` shorthands such as `..Default::default()` |
| `unbounded_channel` | `warn` | flag known unbounded channel constructors |
| `unbounded_collection_growth` | `warn` | flag loop-driven collection growth without reservation or a local bound |
| `assertion_density` | `allow` | flag large functions with too few assertions |
| `raw_arithmetic_overflow` | `allow` | flag unchecked `+`, `-`, or `*` on unsigned integers |
| `sentinel_fallback` | `warn` | flag `<int>::MAX` / `<int>::MIN` sentinel fallbacks instead of explicit branching or tagged fallback text |
| `public_unsafe_api` | `warn` | flag public Rust `unsafe fn` APIs that leak unsafe obligations to callers |
| `defaulting_trait_method` | `allow` | flag trait methods with default bodies that can weaken substitution contracts |
| `discarded_sync_result` | `warn` | flag discarded synchronization and configured commit results in declared durability boundaries |
| `flush_only_durability` | `warn` | flag declared durable boundaries that observe only `Write::flush()` |
| `retry_sync_after_error` | `warn` | flag same-receiver synchronization retries after a handled failure without an admitted recovery operation |

For `public_unsafe_api`, either add a safe wrapper or document the exception with an unsafe-boundary record.

Scope notes for the safety lints:

- `no_unwrap` covers method calls, UFCS calls such as `Option::expect(value, "...")`, and function paths such as `map(Result::unwrap)`. The test-context, known-constructor, and same-branch guard exemptions apply to the call forms; a bare function path is exempt only in test contexts.
- `no_recursion` follows self-calls into closures and `async fn` bodies inside the function, but not into nested items.
- `unchecked_division` checks `/`, `%`, `/=`, and `%=`, including divisors of type `&u64` and similar references; a guard on `*divisor` counts for `divisor`. Inside a `match`, an arm guard that proves the divisor nonzero (`d if d != 0 => total / d`) counts as a guard, and so does an earlier unguarded arm that matches `0` when the divisor is the scrutinee or a plain binding of it (`0 => None, d => Some(total / d)`). An earlier arm for another value, such as `1`, does not.
- `compound_assertion` reads the top-level `&&` or `||` from the parsed `assert!` condition, so a double reference such as `assert!(item == &&expected)` is not flagged. Its split cuts the source at those operators only. When a clause holds a `//` or `/* */` comment, the assertion is still flagged but no split is offered, because a trailing line comment would swallow the closing `)` of the rewritten `assert!`.
- `catch_all_on_enum` skips `Option` and `Result`, which cannot gain variants. Its suggestion lists only the variants that no earlier unguarded arm matches in full; a variant matched only for some field values, such as `Access::Revoke { reason: 0 }`, stays in the list.
- `sentinel_fallback` matches by the resolved constant, so `Ticks::MAX` through `type Ticks = u64` and `Self::MAX` in an integer impl are flagged.
- `unjustified_no_todo_allow` audits `allow` and `expect` of `no_todo` on every node rustc applies them to: items, blocks, `let` and expression statements, closures, match arms, and crate-level `#![allow(no_todo)]`. A justified `#[expect(no_todo)]` still lets `no_todo` fire so the expectation is fulfilled.
- `unbounded_channel` renames only the constructor segment when it is written under its original name and its module prefix exports the bounded constructor. Otherwise (a bare imported name, a local re-export module, or an alias) it spells the bounded constructor through the crate path of a `use` of the constructor, or offers no rewrite.
- `drop_must_use` ignores a type's own `#[must_use]` when the type has a destructor, so `drop(guard)` on `MutexGuard` or `RwLock` guards stays clean. `Result`, must-use types without a destructor, and `#[must_use]` producers are still flagged.
- `unchecked_narrowing` treats a `usize` or `isize` source as 64 bits wide on every target, so `v.len() as u32` and `index as i8` are flagged unless guarded. Findings do not depend on the build target. Casts to `usize` or `isize` belong to `platform_dependent_cast`. Its `try_from` suggestion handles failure in the same way as `platform_dependent_cast`, based on the return type of the enclosing function, closure, or async body: `?` for a `Result` whose error converts from the cast error, `.ok()?` for `Option`, and a placeholder otherwise.
- `raw_arithmetic_overflow` checks `+=`, `-=`, and `*=` on an unsigned place as well as `+`, `-`, and `*`. The compound suggestion rewrites `place op= value` to `place = (place).saturating_op(value)`. A place that would be evaluated twice, such as `values[i]`, gets no suggestion. When the left operand is a constant such as an unsuffixed literal, the suggestion names the result type: `u64::saturating_mul(2, n)`.
- `unbounded_collection_growth` counts a guard joined with `||` when either side bounds the collection, and a guard joined with `&&` only when both sides do. For example, `if items.len() >= limit || shutdown() { break; }` bounds a later push, but `if items.len() >= limit && keep_going() { break; }` does not.
- `discarded_sync_result`, `flush_only_durability`, and `retry_sync_after_error` run only inside consumer-declared boundary functions from `durability_profile_path`. They resolve `sync_all`, `sync_data`, `Write::flush`, configured commit surfaces, and configured adapter observations by definition path, compare only visible receiver text in the analyzed block, and leave aliasing, interprocedural control flow, runtime ordering, and macro-expanded calls unanalyzed. Passing them proves selected source shapes only, never filesystem, device, or hardware durability.
- `assertion_density` uses the Rust lexer to strip comments, so `"/*"` inside a string is not read as a comment start. It skips `#[test]` functions and items whose `cfg` can only hold under `cfg(test)`. Items under `cfg(not(test))` or `cfg(any(test, unix))` are still checked.

## Naming lints (`tigerstyle::naming`)

| Lint | Default | Summary |
| --- | --- | --- |
| `acronym_style` | `warn` | prefer `RpcHandler` over `RPCHandler` |
| `bool_naming` | `warn` | require predicate-style boolean names like `is_ready` |
| `negated_predicate` | `warn` | avoid names like `is_not_ready` or `no_capacity` |
| `numeric_units` | `allow` | require nominal units or suffixes such as `_ms`, `_secs`, or `_bytes` for raw numbers |
| `underscore_in_module_filename` | `warn` | prefer responsibility-based module directories over compound filenames |
| `non_trait_imports` | `warn` | keep concrete owner types qualified at use sites; import traits only for method resolution |
| `renamed_imports` | `warn` | avoid `use ... as ...` aliases that hide searchable item names |
| `path_segment_repetition` | `warn` | avoid repeating crate or module path words in item names |
| `float_for_currency` | `allow` | avoid `f64` in price, cost, or budget fields and function return types |

## Performance lints (`tigerstyle::performance`)

| Lint | Default | Summary |
| --- | --- | --- |
| `clone_then_borrow` | `warn` | flag cloning a value only to immediately take a shared borrow |
| `allocation_in_loop` | `allow` | flag obvious allocation constructors inside loop bodies |

## Structure lints (`tigerstyle::structure`)

| Lint | Default | Summary |
| --- | --- | --- |
| `function_length` | `warn` | flag functions longer than the configured line budget |
| `magic_number` | `warn` | require named constants or config fields for policy thresholds and domain values |
| `excessive_file_length` | `warn` | split oversized source files along module or responsibility boundaries |
| `module_file_count` | `warn` | split crowded module directories into smaller responsibility-based submodules |
| `nested_conditionals` | `warn` | flag deep `if` / `match` nesting |
| `platform_dependent_cast` | `warn` | flag casts like `as usize` from fixed-width integers |
| `usize_in_public_api` | `warn` | avoid `usize` / `isize` at inherent public API boundaries |
| `borrowed_argument_types` | `warn` | prefer `&str`, `&[T]`, and `&Path` over borrowed owned containers |
| `deref_polymorphism` | `warn` | flag public `Deref` impls to nominal target types that hide API boundaries |
| `too_many_parameters` | `warn` | flag signatures that exceed the configured parameter limit |
| `raw_bool_parameter` | `warn` | avoid direct `bool` parameters in public APIs and multiple positional booleans in private callables |
| `raw_bool_parameter_suppression` | `warn` | reject broad or unjustified raw-boolean parameter suppressions |
| `fat_trait` | `warn` | flag traits exceeding the configured method budget |
| `concrete_construction_in_core` | `warn` | flag concrete construction calls in declared core source scopes |
| `type_accumulation` | `allow` | flag excessive inherent impl blocks or module public surface |
| `compound_condition` | `allow` | flag conditions with 3 or more boolean operators |
| `unjustified_allow` | `allow` | require a local `//` comment or a non-empty `reason` on `allow` / `expect` of Clippy or Octet lints |
| `ambiguous_params` | `allow` | flag repeated same-type primitive or raw boundary parameters |
| `primitive_domain_alias` | `allow` | inventory registered domain aliases that still resolve to primitive storage |
| `raw_domain_value` | `allow` | flag registered raw domain values in declared core fields and API signatures |
| `newtype_invariant_bypass` | `allow` | flag public fields and conversions that bypass refined newtype invariants |

See [`boolean-api-modeling.md`](boolean-api-modeling.md) for domain type
choices, generated/external-trait exclusions, narrow expectations, and
independent Clippy co-reporting. `raw_bool_parameter_suppression` governs
exception attrs, not parameter types. Both warnings can be promoted to deny
by strict profiles and the full deny-all gate; no baseline or warning budget
is accepted as migration completion.

## Purity lints (`tigerstyle::purity`)

| Lint | Default | Summary |
| --- | --- | --- |
| `ambient_clock` | `warn` | keep wall-clock reads at explicit shell boundaries |
| `ambient_env` | `warn` | keep process environment access at explicit shell boundaries |
| `ambient_random` | `warn` | keep randomness and generated IDs explicit or shell-owned |
| `global_state_in_core` | `warn` | flag global mutable or lazy state in configured core scopes |
| `impure_call_in_core` | `warn` | flag direct platform-effect calls in configured core scopes |
| `interior_mutability_in_core` | `warn` | flag cells, locks, atomics, and unsafe cells in configured core scopes |
| `logging_in_core` | `warn` | flag logging and console output in configured core scopes |
| `missing_const_fn` | `allow` | identify private free helpers in a conservative primitive HIR subset that appear eligible for `const fn`; calls, methods, closures, and type-dependent operators remain excluded |
| `mutating_input_in_pure` | `warn` | flag mutating input capabilities in declared pure contexts |
| `nondeterministic_output` | `allow` | flag unordered hash iteration collected into ordered output |
| `pure_attr_contract` | `warn` | enforce direct-effect checks on marked pure functions |
| `resource_return_from_core` | `warn` | flag concrete resource-handle return types in configured core scopes |
| `spawn_in_core` | `warn` | flag direct thread or task spawning in configured core scopes |
| `zero_alloc_attr_contract` | `warn` | enforce direct allocation-site checks on marked zero-alloc functions |
| `verified_purity` | `deny` | ban async, IO, and wall-clock usage in `verified/` modules |
| `contradictory_time` | `warn` | avoid taking time as input and then reading the clock again |

A function or closure is marked pure or zero-alloc only by its own attribute whose path ends in `pure` or `zero_alloc` (`#[octet::pure]`, `#[tigerstyle::zero_alloc]`). A marker on a nested function or closure marks only that item, and marker text in a string, comment, or `#[doc = ...]` attribute marks nothing. `pure_attr_contract`, `mutating_input_in_pure`, and `zero_alloc_attr_contract` all use this rule.

`ambient_clock` counts every wall-clock read outside test code: calls such as `Instant::now()` or `Utc::now()`, a clock `now` passed as a function value such as `or_insert_with(Instant::now)`, and `Instant::elapsed` or `SystemTime::elapsed`, which read the clock internally. Comparing injected instants, as in `end.duration_since(start)`, is not a clock read.

## Concurrency lints (`tigerstyle::concurrency`)

| Lint | Default | Summary |
| --- | --- | --- |
| `multi_lock_ordering` | `allow` | require an ordering comment when a struct owns multiple locks |
| `ignored_join_handle` | `warn` | flag discarded task or thread join handles |

## Related docs

- `consumer-guide.md` — setup and rollout entry points
- `trust-boundary-evidence.md#checked-contract-ladder` — how lint findings become checked-contract ladder stages without becoming formal proof
- `rust-idiom-guidance.md#unsafe-trusted-boundary-records` — safe-wrapper and documented-exception paths for `public_unsafe_api`, FFI, raw-pointer, and lifetime-erasing boundaries
- `downstream-rollout.md` — rollout profiles and Aspen-backed evidence
- `../templates/dylint.toml` — copyable consumer template with rollout comments
