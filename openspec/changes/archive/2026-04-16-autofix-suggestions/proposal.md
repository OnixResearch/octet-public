## Why

None of the currently shipped tigerstyle lints provide structured fix
suggestions today. When a
downstream consumer runs tigerstyle and gets 50 warnings, every fix is
manual even when tigerstyle already knows an approved next edit.

This change adds structured suggestions with honest applicability tiers, so
safe cases can participate in `cargo fix` while riskier cases still surface a
reviewable suggested edit.

## What Changes

Add structured suggestions to the first autofix rollout and classify each
pattern with the right `Applicability` tier.

### Machine-applicable (safe to auto-apply)
- `compound_assertion`: split validated top-level `assert!(a && b)` cases into
  separate `assert!` calls

### Review-required (`MaybeIncorrect`)
- `bool_naming`: declaration-site rename suggestion unless the implementation
  also rewrites every affected use site needed to keep the validated fixture
  compiling
- `acronym_style`: declaration-site rename suggestion unless the implementation
  also rewrites every affected use site needed to keep the validated fixture
  compiling
- `unchecked_division`: suggest `assert!(b != 0);` before the division
- `raw_arithmetic_overflow`: suggest checked or saturating arithmetic
- `unchecked_narrowing`: suggest `u32::try_from(value)?`
- `platform_dependent_cast`: suggest `usize::try_from(value)?`

### Placeholder-driven (`HasPlaceholders`)
- `numeric_units`: suggest a unit-bearing rename with explicit placeholder
  choices
- `unjustified_allow`: insert a stub justification comment that the maintainer
  must replace with a real rationale

### Not auto-fixable (diagnostics only)
- `assertion_density`, `function_length`, `ambiguous_params`,
  `multi_lock_ordering`, `catch_all_on_enum`, `no_recursion` — these
  require structural refactoring

### Unchanged / out of scope in this change
- `ambient_clock`, `compound_condition`, `contradictory_time`,
  `explicit_defaults`, `float_for_currency`, `ignored_result`,
  `negated_predicate`, `nested_conditionals`, `no_panic`, `no_unwrap`,
  `too_many_parameters`, `unbounded_channel`, `unbounded_collection_growth`,
  `unbounded_loop`, `unjustified_no_panic_allow`, `usize_in_public_api`,
  `verified_purity` — these keep their current diagnostic-only behavior and do
  not gain structured suggestions in this change

## Non-Goals

- No new lint families or lint-level policy changes
- No `rustfix`-specific acceptance rail beyond the saved nix-devshell `cargo fix`
  proof path
- No declaration + use-site rename propagation for `bool_naming` or
  `acronym_style` in this rollout
- No widening of `compound_assertion` coverage from existing `assert!` handling
  to `debug_assert!` in this rollout
- No structural auto-fixes for diagnostics-only lints like
  `function_length`, `assertion_density`, or `multi_lock_ordering`

## Capabilities

### New Capabilities
- `autofix`: Selected lints emit structured suggestions with explicit
  applicability tiers; only the validated `MachineApplicable` subset is in
  scope for automatic application

## Impact

- **Files**: Every lint source file that gets a suggestion added
- **APIs**: No public API changes
- **Dependencies**: None
- **Testing**: Update ignored UI `.stderr` snapshots for all rollout lints,
  add `.fixed` proof for each `MachineApplicable` pattern through the ignored
  `autofix_machine_applicable` rail, and save one nix-devshell `cargo fix`
  transcript as the supported autofix validation path
