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

The lint MUST be able to warn on `allow` and `expect` attributes, outer
(`#[allow(...)]`) and inner (`#![allow(...)]`), that suppress a governed lint
without a local justification. Governed lint paths are `clippy::*`,
`tigerstyle::*` (Octet lints and Octet groups such as `tigerstyle::safety`),
and Octet's bare compatibility aliases such as `no_unwrap`. rustc's own lints
are not governed, and `no_todo` allows are audited by
`unjustified_no_todo_allow` instead. The lint MUST check attributes on the
crate root, items, associated items, statements, fields, and every other
lint-level node, and MUST decide scope from the attribute's lint paths only,
never from the source file path.

Under a selected strict profile the lint MUST also govern a source `warn`
attribute that reaches one of the profile's required lints, by bare alias,
canonical name, or an Octet group with a required member, because it lowers
that lint from the profile's `deny`. A `warn` that reaches no required lint,
and every `warn` without a profile, MUST NOT be governed. The
suppression-governance lints themselves are covered by the profile's
governance guard instead. A required lint that a later `deny` or `forbid`
attribute on the same node raises back is not lowered, so neither this lint
nor the guard counts it.

The lint MUST accept a non-doc `//` comment on the attribute's source lines,
including a trailing comment after the attribute, or on the line immediately
preceding the attribute. Doc comments (`///`, `//!`) MUST NOT count, and
neither MUST a comment starting with the `TODO(octet)` placeholder that the
lint's own suggestion inserts.

The lint MUST also accept a non-empty `reason = "..."` argument inside the
attribute (stable `lint_reasons` syntax since Rust 1.81) as the
justification. An empty or whitespace-only reason MUST NOT count.

The lint MUST NOT fire on `#[allow(dead_code)]` in `#[cfg(test)]` blocks.

The lint MUST govern only attributes the user wrote in source, including an
allow expanded from `cfg_attr(..., allow(...))`, an allow inside a local
`macro_rules!` body, an allow forwarded through a local macro's `meta` or
`ident` fragment, and an attribute with comments inside it. Whether the user
wrote an attribute MUST be decided from its name token. It MUST NOT fire on
attributes that a macro from another crate generates, including proc-macro
output such as the `#[allow(clippy::...)]` block `#[tokio::main]` emits with
spans borrowed from user tokens. A forwarded attribute is reported at its
name, without an autofix suggestion.

The lint MUST also check, where a macro is defined, each lint-level attribute
written literally in a `macro_rules!` transcriber or `macro` body, including
each one a literal `cfg_attr(predicate, ...)` there would expand into, with the
same scope, justification, and later-raise rules, whether or not the macro is
invoked in that crate. This covers `#[macro_export]` macros whose expansions
land in other crates, where those crates treat the attribute as another
crate's macro output. Attributes built from fragments, such as `#[$meta]` or
`#[$level(...)]`, and a matcher's tokens are left to the expansion-site
check. An attribute in a local macro body MUST be reported once, however
often the macro expands, whether the definition-site or the expansion-site
check reports it. A macro definition that expanding another crate's macro
produces is that crate's code: this lint and the profile guard MUST NOT check
its body in the expanding crate. In a body, a `///` doc comment between
attributes MUST NOT end their attribute list, and a lowering attribute that a
`$(...)` repetition follows in its list MUST be left to the expansion-site
check, since the repetition may expand into a later raise.

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

#### Scenario: Allow with reason argument

- GIVEN `#[allow(clippy::needless_return, reason = "early return mirrors the protocol state table")]`
  with no preceding comment
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Octet lint, alias, and group allows are governed

- GIVEN `#[allow(tigerstyle::no_unwrap)]`, `#[allow(no_unwrap)]`, or
  `#[allow(tigerstyle::safety)]` with no justification
- WHEN the lint runs
- THEN it warns on each attribute

#### Scenario: Doc comment or placeholder is not a justification

- GIVEN `#[allow(clippy::needless_return)]` directly below a `///` doc comment
  or below `// TODO(octet): explain why this allow is safe here`
- WHEN the lint runs
- THEN it warns

#### Scenario: Trailing comment justifies the attribute

- GIVEN `#[allow(clippy::needless_return)] // Early return mirrors the protocol state table.`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: rustc lint allow in a path containing `clippy`

- GIVEN `#[allow(dead_code)]` in a file whose path contains `clippy`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Platform-Dependent Cast

The lint MUST warn on `as usize` and `as isize` casts from fixed-width
integer types (`u32`, `u64`, `i32`, `i64`, etc.).

The lint MUST suggest keeping the fixed-width type or using an explicit
conversion that handles the platform difference. The suggested conversion
MUST fit the enclosing body: `<target>::from(x)` for conversions std
provides losslessly on every platform, `<target>::try_from(x)?` only when the
body returns a `Result` whose error converts from the `TryFrom` error,
`<target>::try_from(x).ok()?` when it returns an `Option`, and a
placeholder form marked `HasPlaceholders` otherwise. Const contexts get no
structured suggestion.

The lint MUST NOT fire on `as usize` from types that are already
platform-sized (e.g., array `.len()` which returns `usize`).

The lint MUST NOT fire inside indexing expressions (`array[x as usize]`)
when `x` is already bounded by a prior check against `array.len()`. The
recognized checks compare the same cast with `array.len()` (`<` or `>`) in
an enclosing `if` condition, the left operand of an enclosing `&&`, or an
earlier `assert!` in an enclosing block. The cast inside such a check is
exempt when it guards a later index of the same place.

This lint exists because Tiger Style mandates explicitly sized types
(`u32`, `i64`, not `usize`) for consistent cross-platform behavior.
clankers' `frame.rs` does `u32::from_be_bytes(len_buf) as usize` — on
a 16-bit platform this truncates. The `as usize` idiom scatters
platform assumptions throughout the codebase.

#### Scenario: Fixed-width to usize

- GIVEN `let size = u32::from_be_bytes(buf) as usize;`
- WHEN the lint runs
- THEN it warns: "`as usize` from u32 is platform-dependent — use explicit conversion"

#### Scenario: Suggestion fits the enclosing body

- GIVEN `fn len(x: u32) -> usize { x as usize }`
- WHEN the lint runs
- THEN it does NOT suggest `usize::try_from(x)?`, which would not compile
- AND it suggests a placeholder `usize::try_from(x)` form marked `HasPlaceholders`

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
such as `pub(crate)` and `pub(super)`. Visibility is the item's effective
visibility: a `pub fn` in a private module that is not re-exported, or a
`pub fn` method on a private type, is not part of the public API and MUST NOT
fire, while crate-root `pub` items and public re-exports still do.

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

The structure lint inventory MUST include an advisory `deref_polymorphism` lint that identifies public `Deref` implementations which make one public local type inherit a nominal target type's API surface. A nominal target is a path that resolves to a struct, enum, union, or type alias definition; generic type parameters, associated types, and `Self` are not nominal targets.

#### Scenario: Public Deref to nominal target warns [r[deref-polymorphism-lint.warn-public-nominal-target]]

- GIVEN a public local wrapper type implements `Deref<Target = Engine>`
- WHEN Tigerstyle analyzes the crate
- THEN `tigerstyle::deref_polymorphism` MUST warn that public `Deref` to a nominal target hides API boundaries and should prefer explicit accessors

#### Scenario: Accepted Deref targets do not warn [r[deref-polymorphism-lint.accepted-targets]]

- GIVEN private wrappers (including `pub` wrappers in private modules that are not re-exported), non-`Deref` impls, `Deref<Target = str>`, `Deref<Target = [T]>`, or a generic guard such as `impl<T> Deref for Guard<'_, T> { type Target = T; }`
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

For this rule, a unit-bearing binding is a binding or constant name ending in an explicit unit suffix already accepted by Octet numeric-unit naming rules. A binding documents only the literal that is its whole initializer (optionally cast or negated); it does not document literals nested deeper in that initializer, such as a threshold inside an `if` condition.

For this rule, a domain-specific wrapper is a constructor call or associated constructor whose type name is not a primitive integer/float type and whose argument is the literal.

The lint MUST NOT warn on conventional low-noise literals where the value is idiomatic and not a policy threshold, including `0`, `1`, and `-1` when they appear in local initialization, increment/decrement, comparison-bound, or indexing-control positions.

The lint MUST NOT warn inside test contexts or generated code contexts already ignored by Octet structural lints.

The lint SHOULD suggest extracting the literal into a named constant or configuration field when the span is suitable for a machine-applicable or maybe-incorrect suggestion.

#### Scenario: Inline timeout threshold [r[magic-number-detection.scenario.inline-timeout-threshold]]

- GIVEN production code with `if elapsed_ms > 30_000 { reconnect(); }`
- WHEN the lint runs
- THEN `magic_number` warns that `30_000` should be named or configured

#### Scenario: Field and method operands name the policy [r[magic-number-detection.scenario.field-and-method-operands]]

- GIVEN production code with `if self.elapsed_ms > 30_000 { reconnect(); }`, `if cfg.retry_attempts > 5 { reconnect(); }`, or `if cfg.retry_limit() > 5 { reconnect(); }`
- WHEN the lint runs
- THEN `magic_number` warns on the literal, because a field or method operand with a unit suffix or domain word names the policy just like a local binding

#### Scenario: Binding does not document a nested threshold [r[magic-number-detection.scenario.binding-nested-threshold]]

- GIVEN production code with `let retry_delay_ms = if elapsed_ms > 30_000 { 50 } else { 10 };`
- WHEN the lint runs
- THEN `magic_number` warns on `30_000`, because the binding names the selected delay rather than the threshold

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

