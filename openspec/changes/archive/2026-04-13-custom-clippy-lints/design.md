# Design: Custom Clippy Lints

## Context

We need a mechanism to run Tiger Style lints during `cargo check` so
violations are caught before code review. The Rust ecosystem offers
three approaches to custom lints: dylint, cargo-marker, and raw
compiler plugins.

## Goals / Non-Goals

**Goals:**

- Each Tiger Style rule maps to one lint with a descriptive name
- Lints run in CI alongside standard `cargo clippy`
- Each lint has fixture-based tests (pass + fail `.rs` files)
- Lint crate is usable by any Rust project, not just aspen
- Suppressible with `#[allow(tigerstyle::lint_name)]`

**Non-Goals:**

- Auto-fix (may come later, but not in v1)
- IDE integration beyond what dylint provides for free
- Linting non-Rust code (markdown, nix, TOML)

## Decisions

### 1. Lint Framework: dylint

**Choice:** `dylint` over `cargo-marker` or raw compiler plugins.

**Rationale:** dylint provides the full rustc lint API (`LateLintPass`,
`EarlyLintPass`) with type information, works with the stable compiler
interface via pinned toolchain, and has proven use in production
(Foundry, Trail of Bits tools). cargo-marker has a cleaner API but is
younger with fewer real-world deployments. Raw compiler plugins are
unstable and break across nightly versions.

**Alternative rejected — cargo-marker:** Simpler API surface, stable
Rust, but the project is pre-1.0 and the lint API doesn't expose full
type resolution (needed for `ignored-result` and `bool-naming`).

**Alternative rejected — pre-commit scripts with syn:** No type info,
can't distinguish `bool` bindings from other types, can't resolve
method return types for Result detection.

### 2. Crate Structure

**Choice:** Single `dylint_linting` library crate with one module per
lint domain.

```
tigerstyle/
├── Cargo.toml
├── flake.nix
├── src/
│   ├── lib.rs            # register_lints, declare_lint_pass
│   ├── safety/
│   │   ├── mod.rs
│   │   ├── compound_assertion.rs
│   │   ├── assertion_density.rs
│   │   ├── unbounded_loop.rs
│   │   └── ignored_result.rs
│   ├── naming/
│   │   ├── mod.rs
│   │   ├── bool_naming.rs
│   │   ├── acronym_style.rs
│   │   └── numeric_units.rs
│   ├── structure/
│   │   ├── mod.rs
│   │   ├── function_length.rs
│   │   ├── compound_condition.rs
│   │   └── ambiguous_params.rs
│   ├── purity/
│   │   ├── mod.rs
│   │   ├── ambient_clock.rs
│   │   ├── verified_purity.rs
│   │   └── contradictory_time.rs
│   └── concurrency/
│       ├── mod.rs
│       └── multi_lock_ordering.rs
├── ui_tests/
│   ├── compound_assertion.rs      # should-fail fixture
│   ├── compound_assertion.stderr  # expected diagnostics
│   ├── bool_naming.rs
│   ├── bool_naming.stderr
│   └── ...
└── openspec/
```

**Rationale:** Mirrors the spec domains. Each lint file is self-contained:
`declare_lint!`, `declare_lint_pass!`, and the `LateLintPass` impl.
A single crate keeps the build simple — dylint loads one `.so`.

### 3. Lint Pass Selection

**Choice:** Use `LateLintPass` (post-type-resolution) for lints that
need type information, `EarlyLintPass` (pre-type-resolution) for
syntax-only lints.

| Lint | Pass | Why |
|------|------|-----|
| compound_assertion | Early | Macro argument is pure syntax |
| assertion_density | Late | Needs to count macro expansions in fn body |
| unbounded_loop | Early | Loop structure is syntactic |
| ignored_result | Late | Needs return type of expression |
| unchecked_narrowing | Late | Needs source/target type widths |
| raw_arithmetic_overflow | Late | Needs operand types to confirm unsigned |
| unchecked_division | Late | Needs to resolve divisor origin (param vs const) |
| bool_naming | Late | Needs type inference result |
| acronym_style | Early | Identifier spelling is syntactic |
| numeric_units | Early | Identifier name is syntactic |
| function_length | Early | Line counting is syntactic |
| compound_condition | Early | Operator count is syntactic |
| ambiguous_params | Late | Needs resolved parameter types |
| verified_purity | Early | Path detection + keyword scan |
| contradictory_time | Late | Needs param names + body call resolution |
| ambient_clock | Late | Needs to resolve direct clock reads and ignore test-only contexts |
| unjustified_allow | Early | Attribute + comment presence is syntactic |
| catch_all_on_enum | Late | Needs to know the match scrutinee is an enum |
| float_for_currency | Late | Needs type resolution to confirm f64/f32 |
| platform_dependent_cast | Late | Needs source type width |
| multi_lock_ordering | Early | Struct field types are syntactic |

### 4. Test Strategy

**Choice:** `dylint_testing::ui_test` for each lint.

Each lint gets a `.rs` file with annotated `//~ WARNING` or `//~ ERROR`
comments and a `.stderr` file with the expected diagnostic output.
`cargo test` runs all fixtures through the lint and diffs output.

Supplementary integration test: run `cargo dylint tigerstyle` against
a snapshot of aspen and verify known violations are detected.

### 5. CI Integration

**Choice:** nix flake check runs `cargo dylint tigerstyle` on the
workspace.

```nix
checks.tigerstyle = pkgs.runCommand "tigerstyle-lint" {
  nativeBuildInputs = [ cargo dylint-link ];
} ''
  cd ${src}
  cargo dylint tigerstyle
  touch $out
'';
```

Aspen adds `tigerstyle` as a flake input and runs it as a check.

### 6. Lint Severity

| Lint | Default Level | Rationale |
|------|---------------|-----------|
| compound_assertion | warn | Fix is mechanical, no risk |
| assertion_density | allow | Advisory — not all functions need assertions |
| unbounded_loop | warn | Real safety issue |
| ignored_result | warn | 92% catastrophic failure stat |
| unchecked_narrowing | warn | Silent truncation causes data corruption (verified-logic built a module for this) |
| raw_arithmetic_overflow | allow | High false positive risk in non-critical code |
| unchecked_division | warn | Division by zero is always a bug |
| bool_naming | warn | Convention, easy to fix |
| acronym_style | warn | Convention, easy to fix |
| numeric_units | allow | Advisory — high false positive risk |
| function_length | warn | Threshold is well-defined |
| compound_condition | allow | Subjective decomposition |
| ambiguous_params | allow | Noisy on generic APIs |
| verified_purity | deny | Hard rule for verified/ modules |
| contradictory_time | warn | Two time sources in one function is always a design error |
| ambient_clock | warn | Wall-clock reads should stay at explicit shell boundaries |
| unjustified_allow | allow | Useful for audits, noisy by default |
| catch_all_on_enum | warn | Silent deny/grant on new variants is a security risk (clankers capability.rs) |
| float_for_currency | allow | Advisory — many codebases use f64 for cost (clankers cost_tracker) |
| platform_dependent_cast | warn | `as usize` from fixed-width violates explicit sizing rule (clankers frame.rs) |
| multi_lock_ordering | allow | Advisory — useful for concurrency audits |

## Risks / Trade-offs

**[dylint toolchain coupling]** dylint requires the lint crate to be
compiled with the exact same rustc version as the target project.
Mitigation: pin `rust-toolchain.toml` in the lint crate and document
the version requirement.

**[False positives on assertion density]** Many functions legitimately
need zero assertions (pure mappers, trivial accessors). Mitigation:
default to `allow`, let projects opt in. Minimum line threshold (20)
filters out trivial functions.

**[Acronym allowlist maintenance]** Some acronyms (`IO`, `ID`) are
idiomatic in Rust. Mitigation: ship a default allowlist, let projects
extend it via config.

**[verified_purity path detection]** Relies on file path containing
`/verified/`. A file named `unverified/helper.rs` won't be caught if
moved. Mitigation: also support an opt-in `#[tigerstyle::verified]`
attribute for explicit marking.
