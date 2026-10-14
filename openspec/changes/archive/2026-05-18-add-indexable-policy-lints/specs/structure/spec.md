## ADDED Requirements

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
