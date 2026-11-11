## Phase 1: Early-pass lint fixtures

- [x] Add `compound_assertion.rs` + `.stderr` (fire on `assert!(a && b)`, skip `assert_eq!`)
- [x] Add `function_length.rs` + `.stderr` (fire on 85-line fn, skip `#[test]`)
- [x] Add `acronym_style.rs` + `.stderr` (fire on `RPCHandler`, skip all-caps)
- [x] Add `numeric_units.rs` + `.stderr` (fire on bare `timeout`, skip suffixed)
- [x] Add `unbounded_loop.rs` + `.stderr` (fire on bare `loop {}`, skip bounded `for`)
- [x] Add `compound_condition.rs` + `.stderr` (fire on 3+ operators, skip simple)
- [x] Add `verified/verified_purity.rs` + `.stderr` (fire on async in `verified/`, skip Duration)
- [x] Add `unjustified_allow.rs` + `.stderr` (fire on bare `#[allow]`, skip commented)
- [x] Add `assertion_density.rs` + `.stderr` (fire on 30-line fn with 0 asserts, skip short)
- [x] Add `multi_lock_ordering.rs` + `.stderr` (fire on 2 Mutex without comment, skip single)
- [x] Add `no_panic.rs` + `.stderr` (fire on `panic!()` in production, skip test)

## Phase 2: Late-pass lint fixtures

- [x] Add `bool_naming.rs` + `.stderr` (fire on `let active: bool`, skip prefixed)
- [x] Add `ignored_result.rs` + `.stderr` (fire on `let _ = fallible()`, skip non-Result)
- [x] Add `unchecked_narrowing.rs` + `.stderr` (fire on bare `as u32`, skip guarded)
- [x] Add `unchecked_division.rs` + `.stderr` (fire on `a / b` param, skip constant)
- [x] Add `contradictory_time.rs` + `.stderr` (fire on param+clock, skip pure)
- [x] Add `catch_all_on_enum.rs` + `.stderr` (fire on `_ => false`, skip commented)
- [x] Add `platform_dependent_cast.rs` + `.stderr` (fire on `as usize`, skip already-platform-sized and non-`usize` casts)
- [x] Add `float_for_currency.rs` + `.stderr` (fire on `cost: f64`, skip `ratio`)
- [x] Add `ambiguous_params.rs` + `.stderr` (fire on 2 same-type, skip mixed)
- [x] Add `raw_arithmetic_overflow.rs` + `.stderr` (fire on raw `*`, skip const)
- [x] Add `no_unwrap.rs` + `.stderr` (fire on `.unwrap()`, skip test)
- [x] Add `no_recursion.rs` + `.stderr` (fire on self-call, skip non-recursive)

## Phase 3: Config-variant fixtures

- [x] Add `function_length_custom/` subdirectory with `dylint.toml` override and fixture
- [x] Add `assertion_density_custom/` subdirectory with adjusted thresholds
- [x] Add `nesting_custom/` subdirectory with `nesting_max_depth = 5` override
- [x] Add `acronym_allowlist_custom/` subdirectory with `acronym_allowlist = ["IO"]` override

## Phase 4: Validation

- [x] Keep explicit per-fixture compiler exit-status checks in the UI harness
- [x] Verify all fixtures pass in `nix flake check` (checks.ui gate)
- [x] Update test documentation noting fixture count and coverage
