# Structure Specification

## Purpose

Tiger Style structural requirements for lint enforcement and agent-facing
skill coverage: function length, compound-condition decomposition,
interface clarity, and consistent formatting.
## Requirements
### Requirement: Function Interface And Data-Handling Coverage In Skill

The Tiger Style skill MUST include the missing structure guidance for
function design and data handling.

The skill MUST cover:

- simplifying function signatures
- constructing large objects in place
- passing large objects by reference
- minimizing dimensionality in parameters and return types
- handling buffer allocation and cleanup in one logical block
- handling division intentionally with explicit rounding intent
- minimizing variable scope

#### Scenario: Reviewing a large by-value parameter

- GIVEN a function takes a large struct by value without ownership need
- WHEN an agent follows the skill
- THEN it is prompted to prefer passing by reference

### Requirement: Formatting And Documentation Norms In Skill

The Tiger Style skill MUST include the missing code-consistency and
communication rules from the reference.

The skill MUST cover:

- consistent indentation
- reasonable line-length limits
- clear code block separation
- consistent bracing rules
- standardized tooling where practical
- standalone commit messages that include the why
- documentation of test methodology, including how a test validates a property

#### Scenario: Writing a test comment

- GIVEN a correctness test for mutual exclusion
- WHEN an agent follows the skill
- THEN it documents how the test establishes the property, not only the property name

### Requirement: Function Length Limit

The lint MUST warn on functions whose body exceeds 70 lines (excluding
blank lines and comment-only lines).

The lint MUST report the actual line count and the 70-line limit.

The lint MUST NOT fire on functions annotated with `#[test]`.

The lint SHOULD NOT fire on generated code (`#[automatically_derived]`,
`include!` bodies, or files in `target/`).

The lint MAY offer a configurable threshold via `#[allow(tigerstyle::function_length)]`
or a config file.

#### Scenario: Long handler function

- GIVEN a function body spanning 85 non-blank, non-comment lines
- WHEN the lint runs
- THEN it warns: "function `handle` is 85 lines (limit: 70)"

#### Scenario: Long function with many comments

- GIVEN a function with 90 total lines but only 65 code lines
- WHEN the lint runs
- THEN it does NOT fire (65 code lines < 70)

#### Scenario: Test function

- GIVEN a 200-line function annotated `#[test]`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Compound Condition Decomposition

The lint MUST be able to warn on `if` conditions containing 3 or more `&&`/`||`
operators at the same nesting level.

The lint MUST suggest decomposing into early returns or nested `if` blocks.

The lint MUST NOT fire on `match` guard expressions.

The lint MUST NOT fire inside `assert!` macros (compound assertions are
covered by a separate lint).

#### Scenario: Triple conjunction

- GIVEN `if a.is_some() && !a.unwrap().expired() && a.unwrap().holder == id`
- WHEN the lint runs
- THEN it suggests decomposing into nested `if let` / early returns

#### Scenario: Simple binary condition

- GIVEN `if x > 0 && x < 100`
- WHEN the lint runs
- THEN it does NOT fire (only 1 operator)

### Requirement: Ambiguous Same-Type Parameters

The lint MUST be able to warn on function signatures with 2 or more consecutive
parameters of the same primitive type (`u8`–`u128`, `i8`–`i128`, `f32`,
`f64`, `bool`, `&str`, `String`).

The lint MUST suggest wrapping in a named struct.

The lint MUST NOT fire on trait method implementations (the signature
is dictated by the trait).

The lint MUST NOT fire on functions with only 1 parameter of that type.

#### Scenario: Two u64 parameters

- GIVEN `fn compute_deadline(now_ms: u64, timeout_ms: u64) -> u64`
- WHEN the lint runs
- THEN it suggests a `DeadlineParams { now_ms: u64, timeout_ms: u64 }` struct

#### Scenario: Mixed types

- GIVEN `fn set(key: &str, value: &[u8], ttl: u64) -> Result<()>`
- WHEN the lint runs
- THEN it does NOT fire (no consecutive same-type params)

### Requirement: Allow Attribute Without Justification

The lint MUST be able to warn on `#[allow(clippy::...)]` and `#![allow(clippy::...)]`
attributes that are not preceded by a comment explaining why the lint
is suppressed.

The lint MUST accept a comment on the same line or the line immediately
preceding the attribute.

The lint MUST NOT fire on `#[allow(dead_code)]` in `#[cfg(test)]` blocks.

The lint MUST NOT fire on workspace-level lint configuration in
`Cargo.toml` (those are outside Clippy's reach anyway).

This lint exists because verified-logic carefully documents every
`#[allow(...)]` — its `conversion.rs` has a module-level `#![allow(...)]`
block with a comment explaining that "verified casts are the module's
entire purpose", and its `Cargo.toml` documents each allowed lint with
a `# ---` section comment. Unjustified suppression hides real bugs.

#### Scenario: Bare allow without comment

- GIVEN `#[allow(clippy::cast_possible_truncation)]` with no preceding comment
- WHEN the lint runs
- THEN it warns: "`#[allow]` without justification — add a comment explaining why"

#### Scenario: Allow with justification

- GIVEN:
  ```rust
  // Verified casts: Verus checks every narrowing in this module.
  #[allow(clippy::cast_possible_truncation)]
  ```
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Platform-Dependent Cast

The lint MUST warn on `as usize` and `as isize` casts from fixed-width
integer types (`u32`, `u64`, `i32`, `i64`, etc.).

The lint MUST suggest keeping the fixed-width type or using an explicit
conversion that handles the platform difference.

The lint MUST NOT fire on `as usize` from types that are already
platform-sized (e.g., array `.len()` which returns `usize`).

The lint MUST NOT fire inside indexing expressions (`array[x as usize]`)
when `x` is already bounded by a prior check against `array.len()`.

This lint exists because Tiger Style mandates explicitly sized types
(`u32`, `i64`, not `usize`) for consistent cross-platform behavior.
clankers' `frame.rs` does `u32::from_be_bytes(len_buf) as usize` — on
a 16-bit platform this truncates. The `as usize` idiom scatters
platform assumptions throughout the codebase.

#### Scenario: Fixed-width to usize

- GIVEN `let size = u32::from_be_bytes(buf) as usize;`
- WHEN the lint runs
- THEN it warns: "`as usize` from u32 is platform-dependent — use explicit conversion"

#### Scenario: Len-bounded index

- GIVEN:
  ```rust
  let idx: u32 = header.index;
  assert!((idx as usize) < data.len());
  let val = data[idx as usize];
  ```
- WHEN the lint runs
- THEN it does NOT fire (bounded by len check)

### Requirement: Usize In Public API

The lint MUST warn on inherent `pub fn` parameters and return types that use
`usize` or `isize`, including nested occurrences inside references, slices,
arrays, tuples, and generic arguments such as `Vec<usize>`.

The lint MUST suggest using fixed-width integer types such as `u32`, `u64`,
`i32`, or `i64`.

The lint MUST NOT fire on trait declarations or trait implementations where the
signature is dictated by the trait, including but not limited to `Index`,
`IndexMut`, `Iterator::nth`, and `Read::read`.

The lint MUST NOT fire on private functions or on restricted-visibility items
such as `pub(crate)` and `pub(super)`.

#### Scenario: Public function with usize parameter

- GIVEN `pub fn get_item(index: usize) -> &Item`
- WHEN the lint runs
- THEN it warns that `usize` in a public API harms cross-platform consistency

#### Scenario: Trait-directed signature

- GIVEN `impl Index<usize> for MyVec { fn index(&self, idx: usize) -> &T { ... } }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Too Many Parameters

The lint MUST warn on named function items with more than a configurable
number of parameters. The default threshold MUST be 5.

`self`, `&self`, and `&mut self` receivers MUST NOT count toward the threshold.

The lint MUST suggest grouping parameters into a named struct.

The lint MUST evaluate free functions, inherent methods, and trait method
signatures. Closures are out of scope.

The lint MUST NOT fire on trait implementations.

The lint MUST NOT fire in the repo's supported test contexts, including
`#[test]` functions and `#[cfg(test)]` items detected by the shared
test-context helper.

The threshold MUST be configurable via `[tigerstyle].max_parameters` in
`dylint.toml`, using the same repo-owned config loading path as other numeric
thresholds.

#### Scenario: Six parameters at default threshold

- GIVEN `fn connect(host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str)`
- WHEN the lint runs
- THEN it warns that the function exceeds the default limit of 5 parameters

#### Scenario: Five parameters at default threshold

- GIVEN a function with exactly 5 counted parameters
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Trait declaration still warns

- GIVEN `trait Connector { fn connect(&self, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8); }`
- WHEN the lint runs
- THEN it warns because the declaration still exposes a long positional API

#### Scenario: Custom threshold

- GIVEN `[tigerstyle] max_parameters = 7` in `dylint.toml`
- WHEN the lint runs on a function with 6 counted parameters
- THEN it does NOT fire

### Requirement: Borrowed Argument Types Lint [r[borrowed-argument-types-lint]]

The lint MUST be able to warn on function and method parameters that take references to owned standard container types when a more general borrowed type communicates the same ownership contract.

The first implementation MUST cover `&String` with a suggestion to use `&str`, `&Vec<T>` with a suggestion to use `&[T]`, and `&PathBuf` with a suggestion to use `&Path`.

The lint MUST remain advisory or allow-by-default until compliant, noncompliant, and false-positive-boundary fixtures prove the intended behavior.

#### Scenario: String reference parameter [r[borrowed-argument-types-lint.scenario.string-reference]]

- GIVEN `fn parse_name(name: &String) -> Result<Name, Error>`
- WHEN the lint runs
- THEN it warns that `&String` exposes an owned container where `&str` is the clearer borrowed argument type
- AND the diagnostic suggests `&str` with review-required applicability

#### Scenario: Vec reference parameter [r[borrowed-argument-types-lint.scenario.vec-reference]]

- GIVEN `fn checksum(bytes: &Vec<u8>) -> u32`
- WHEN the lint runs
- THEN it warns that `&Vec<u8>` should be accepted as `&[u8]` when ownership and vector capacity are not part of the contract

#### Scenario: PathBuf reference parameter [r[borrowed-argument-types-lint.scenario.pathbuf-reference]]

- GIVEN `fn load_config(path: &PathBuf) -> Result<Config, Error>`
- WHEN the lint runs
- THEN it warns that `&PathBuf` should be accepted as `&Path` at borrowed path boundaries

#### Scenario: Trait dictated signature is skipped [r[borrowed-argument-types-lint.scenario.trait-dictated-skip]]

- GIVEN a trait method or trait implementation requires a parameter such as `fn visit(&self, value: &String)`
- WHEN the lint runs
- THEN it does NOT warn because the implementer may not control the signature

#### Scenario: Extern ABI signature is skipped [r[borrowed-argument-types-lint.scenario.extern-skip]]

- GIVEN an `extern "C"` function or callback boundary with a fixed ABI signature
- WHEN the lint runs
- THEN it does NOT warn because the signature belongs to an external boundary contract

#### Scenario: Already borrowed forms are accepted [r[borrowed-argument-types-lint.scenario.borrowed-forms-accepted]]

- GIVEN parameters typed as `&str`, `&[u8]`, and `&Path`
- WHEN the lint runs
- THEN it does NOT warn

### Requirement: Deref Polymorphism Lint [r[deref-polymorphism-lint]]

The structure lint inventory MUST include an advisory `deref_polymorphism` lint that identifies public `Deref` implementations which make one public local type inherit a nominal target type's API surface.

#### Scenario: Public Deref to nominal target warns [r[deref-polymorphism-lint.warn-public-nominal-target]]

- GIVEN a public local wrapper type implements `Deref<Target = Engine>`
- WHEN Tigerstyle analyzes the crate
- THEN `tigerstyle::deref_polymorphism` MUST warn that public `Deref` to a nominal target hides API boundaries and should prefer explicit accessors

#### Scenario: Accepted Deref targets do not warn [r[deref-polymorphism-lint.accepted-targets]]

- GIVEN private wrappers, non-`Deref` impls, `Deref<Target = str>`, or `Deref<Target = [T]>`
- WHEN Tigerstyle analyzes the crate
- THEN `tigerstyle::deref_polymorphism` MUST NOT warn for those accepted borrowed-view or non-public cases

#### Scenario: Inventory and rollout metadata stay synchronized [r[deref-polymorphism-lint.inventory]]

- GIVEN the lint is implemented
- WHEN docs, templates, diagnostics, and fixture manifests are checked
- THEN the lint catalog MUST list `deref_polymorphism` as a default-warn structure lint and rollout examples MUST keep it advisory until future promotion evidence exists

### Requirement: Magic Number Detection [r[magic-number-detection]]

The system MUST provide a warn-by-default structure lint named `magic_number`.

The lint MUST warn on inline numeric literals in production code when the literal is not self-documenting through a named constant, unit-bearing binding, or domain-specific wrapper.

For this rule, a named constant is a `const` or `static` item whose identifier contains at least one non-generic domain word beyond the numeric type or unit.

For this rule, a unit-bearing binding is a binding or constant name ending in an explicit unit suffix already accepted by Octet numeric-unit naming rules.

For this rule, a domain-specific wrapper is a constructor call or associated constructor whose type name is not a primitive integer/float type and whose argument is the literal.

The lint MUST NOT warn on conventional low-noise literals where the value is idiomatic and not a policy threshold, including `0`, `1`, and `-1` when they appear in local initialization, increment/decrement, comparison-bound, or indexing-control positions.

The lint MUST NOT warn inside test contexts or generated code contexts already ignored by Octet structural lints.

The lint SHOULD suggest extracting the literal into a named constant or configuration field when the span is suitable for a machine-applicable or maybe-incorrect suggestion.

#### Scenario: Inline timeout threshold [r[magic-number-detection.scenario.inline-timeout-threshold]]

- GIVEN production code with `if elapsed_ms > 30_000 { reconnect(); }`
- WHEN the lint runs
- THEN `magic_number` warns that `30_000` should be named or configured

#### Scenario: Named constant [r[magic-number-detection.scenario.named-constant]]

- GIVEN production code with `const RECONNECT_TIMEOUT_MS: u64 = 30_000; if elapsed_ms > RECONNECT_TIMEOUT_MS { reconnect(); }`
- WHEN the lint runs
- THEN `magic_number` does NOT fire at the use site

#### Scenario: Conventional loop literals [r[magic-number-detection.scenario.conventional-literals]]

- GIVEN production code with `let mut i = 0; i += 1; if i > -1 { work(i); }`
- WHEN the lint runs
- THEN `magic_number` does NOT fire on `0`, `1`, or `-1`

#### Scenario: Unit-bearing binding [r[magic-number-detection.scenario.unit-bearing-binding]]

- GIVEN production code with `let retry_delay_ms = 250; sleep_ms(retry_delay_ms);`
- WHEN the lint runs
- THEN `magic_number` does NOT fire on `250`

#### Scenario: Test-context magic number [r[magic-number-detection.scenario.test-context-suppressed]]

- GIVEN code inside `#[test]` with `assert_eq!(timeout_ms(), 30_000)`
- WHEN the lint runs
- THEN `magic_number` does NOT fire

### Requirement: Excessive File Length Detection [r[excessive-file-length-detection]]

The system MUST provide a warn-by-default structure lint named `excessive_file_length`.

The lint MUST warn when a non-generated source file exceeds a configurable line-count limit.

The default limit MUST be 300 physical lines.

The lint MUST count the whole source file, not only function bodies.

The lint MUST NOT warn for generated files, build output under `target/`, or UI/test fixture files when those paths are being linted as fixture inputs.

#### Scenario: Oversized source file [r[excessive-file-length-detection.scenario.oversized-source-file]]

- GIVEN a production source file with 350 physical lines
- WHEN the lint runs
- THEN `excessive_file_length` warns with the observed line count and the configured limit

#### Scenario: Source file within limit [r[excessive-file-length-detection.scenario.within-limit]]

- GIVEN a production source file with 220 physical lines
- WHEN the lint runs
- THEN `excessive_file_length` does NOT fire

### Requirement: Module File Count Detection [r[module-file-count-detection]]

The system MUST provide a warn-by-default structure lint named `module_file_count`.

The lint MUST warn when one module directory contains more Rust source files than a configurable limit.

The default limit MUST be 7 `.rs` files per module directory.

The lint MUST count module-local `.rs` files deterministically using source-map/file-system paths available during lint execution.

The lint MUST NOT count generated build output, `target/`, or files outside the linted package root.

#### Scenario: Module directory over limit [r[module-file-count-detection.scenario.over-limit]]

- GIVEN a module directory containing 9 Rust source files
- WHEN the lint runs
- THEN `module_file_count` warns with the observed file count and configured limit

#### Scenario: Module directory at limit [r[module-file-count-detection.scenario.at-limit]]

- GIVEN a module directory containing exactly 7 Rust source files
- WHEN the lint runs
- THEN `module_file_count` does NOT fire

