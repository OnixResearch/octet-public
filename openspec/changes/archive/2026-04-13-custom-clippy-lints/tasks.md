# Tasks: Custom Clippy Lints

## Phase 1: Scaffolding

- [x] Initialize Rust crate with `dylint_linting` dependency and nix flake ✅ 45m (started: 2026-03-23T00:44Z → completed: 2026-03-23T01:29Z)
- [x] Set up `ui_tests/` directory with `dylint_testing` harness ✅ 10m (started: 2026-03-23T00:50Z → completed: 2026-03-23T01:00Z)
- [x] Create `src/lib.rs` with lint registration skeleton ✅ 5m (started: 2026-03-23T00:48Z → completed: 2026-03-23T00:53Z)
- [x] Create module structure: `safety/`, `naming/`, `structure/`, `purity/`, `concurrency/` ✅ 5m (started: 2026-03-23T00:50Z → completed: 2026-03-23T00:55Z)
- [x] Verify `cargo test` runs and passes with zero lints registered ✅ 5m (started: 2026-03-23T01:25Z → completed: 2026-03-23T01:30Z)

## Phase 2: Tier 1 Lints (syntax-only, Early pass)

- [x] `compound_assertion` — detect `assert!(a && b)`, suggest split ✅ 20m (started: 2026-03-23T01:35Z → completed: 2026-03-23T01:55Z)
- [x] `function_length` — detect functions over 70 non-blank/non-comment lines ✅ 15m (started: 2026-03-23T01:35Z → completed: 2026-03-23T01:50Z)
- [x] `acronym_style` — detect `RPCHandler` style, suggest `RpcHandler` ✅ 15m (started: 2026-03-23T01:35Z → completed: 2026-03-23T01:50Z)
- [x] `numeric_units` — detect bare `timeout`, `size` without unit suffix ✅ 10m (started: 2026-03-23T01:40Z → completed: 2026-03-23T01:50Z)

## Phase 3: Tier 1 Lints (type-aware, Late pass)

- [x] `bool_naming` — detect bool bindings without `is_`/`has_`/`should_`/`can_` prefix ✅ 10m (started: 2026-03-23T02:00Z → completed: 2026-03-23T02:10Z)
- [x] `ignored_result` — detect `let _ = fallible_expr()` discarding Result ✅ 10m (started: 2026-03-23T02:00Z → completed: 2026-03-23T02:10Z)

## Phase 4: Tier 2 Lints (verified-logic patterns, Late pass)

- [x] `unchecked_narrowing` — detect bare `as u32`/`u16`/`u8` without bounds check ✅ 10m (started: 2026-03-23T02:12Z → completed: 2026-03-23T02:22Z)
- [x] `unchecked_division` — detect `a / b` where `b` is param without zero guard ✅ 10m (started: 2026-03-23T02:12Z → completed: 2026-03-23T02:22Z)
- [x] `contradictory_time` — detect time param + `Instant::now()` in same function ✅ 15m (started: 2026-03-23T02:12Z → completed: 2026-03-23T02:27Z)

## Phase 5: Tier 2 Lints (structure)

- [x] `unbounded_loop` — detect `loop {}` without iteration bound ✅ 10m (started: 2026-03-23T02:30Z → completed: 2026-03-23T02:40Z)
- [x] `compound_condition` — detect `if` with 3+ boolean operators ✅ 10m (started: 2026-03-23T02:30Z → completed: 2026-03-23T02:40Z)
- [x] `verified_purity` — detect async/IO/time in `verified/` modules ✅ 15m (started: 2026-03-23T02:30Z → completed: 2026-03-23T02:45Z)
- [x] `unjustified_allow` — detect `#[allow(clippy::...)]` without comment ✅ 15m (started: 2026-03-23T02:30Z → completed: 2026-03-23T02:45Z)

## Phase 6: Tier 2 Lints (clankers patterns)

- [x] `catch_all_on_enum` — detect `_ => false`/`_ => None` catch-all on enum match ✅ 15m (started: 2026-03-23T02:50Z → completed: 2026-03-23T03:05Z)
- [x] `platform_dependent_cast` — detect `as usize`/`as isize` from fixed-width types ✅ 10m (started: 2026-03-23T02:50Z → completed: 2026-03-23T03:00Z)
- [x] `float_for_currency` — detect f64/f32 in cost/price/budget struct fields ✅ 10m (started: 2026-03-23T02:50Z → completed: 2026-03-23T03:00Z)

## Phase 7: Tier 3 Lints

- [x] `assertion_density` — warn on 20+ line functions with < 2 assertions ✅ 10m (started: 2026-03-23T02:55Z → completed: 2026-03-23T03:05Z)
- [x] `ambiguous_params` — detect 2+ consecutive same-type primitive params ✅ 15m (started: 2026-03-23T02:55Z → completed: 2026-03-23T03:10Z)
- [x] `raw_arithmetic_overflow` — detect unprotected `+`/`-`/`*` on unsigned types ✅ 10m (started: 2026-03-23T02:55Z → completed: 2026-03-23T03:05Z)
- [x] `multi_lock_ordering` — detect structs with 2+ Mutex/RwLock without ordering comment ✅ 10m (started: 2026-03-23T02:55Z → completed: 2026-03-23T03:05Z)

## Phase 8: Integration

- [x] Run against aspen workspace, triage results ✅ 20m (started: 2026-03-23T03:45Z → completed: 2026-03-23T04:05Z)
- [x] Run against verified-logic, confirm zero false positives on verified code ✅ 15m (started: 2026-03-23T03:45Z → completed: 2026-03-23T04:00Z)
- [x] Run against clankers workspace, triage results ✅ 15m (started: 2026-03-23T03:45Z → completed: 2026-03-23T04:00Z)
- [x] Add default severity levels from design doc ✅ 5m (started: 2026-03-23T03:15Z → completed: 2026-03-23T03:20Z)
- [x] Write README with usage instructions and lint catalog ✅ 15m (started: 2026-03-23T03:15Z → completed: 2026-03-23T03:30Z)
- [x] Add tigerstyle invocation to Aspen's equivalent check path (`scripts/tigerstyle-check.sh`)
- [x] Configure Aspen `dylint.toml` thresholds and allowlists for known exceptions
