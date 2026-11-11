## ADDED Requirements

### Requirement: Must-Use Drop Detection [r[drop-must-use-detection]]

The system MUST provide a warn-by-default safety lint named `drop_must_use`.

The lint MUST warn when user code explicitly calls `drop(expr)` where `expr` has a must-use obligation, including:

- values whose type carries `#[must_use]`, such as `Result`
- calls whose callee return value carries `#[must_use]`

The lint MUST NOT warn for ordinary non-must-use values.

The lint MUST NOT warn in the repo's supported test contexts, including `#[cfg(test)]`, `#[test]`, and shared file/context heuristics already used by Octet safety lints.

#### Scenario: Dropped Result [r[drop-must-use-detection.scenario.dropped-result]]

- GIVEN production code that calls `drop(write_record())` where `write_record()` returns `Result<(), Error>`
- WHEN the lint runs
- THEN `drop_must_use` warns that the must-use value is being explicitly discarded

#### Scenario: Dropped non-must-use value [r[drop-must-use-detection.scenario.non-must-use]]

- GIVEN production code that calls `drop(buffer)` where `buffer` has no must-use obligation
- WHEN the lint runs
- THEN `drop_must_use` does NOT fire

#### Scenario: Dropped must-use return value [r[drop-must-use-detection.scenario.must-use-return-value]]

- GIVEN production code that calls `drop(build_token())` where `build_token()` is annotated `#[must_use]`
- WHEN the lint runs
- THEN `drop_must_use` warns that the callee return value is explicitly discarded

#### Scenario: Test-context dropped Result [r[drop-must-use-detection.scenario.test-context-suppressed]]

- GIVEN code inside `#[test]` that calls `drop(write_record())` where `write_record()` returns `Result<(), Error>`
- WHEN the lint runs
- THEN `drop_must_use` does NOT fire

### Requirement: Fallible Integer Fallback Detection [r[fallible-int-fallback-detection]]

The system MUST provide a warn-by-default safety lint named `fallible_int_fallback`.

The lint MUST warn when code hides a failed integer conversion by applying a fallback to a `Result<integer, TryFromIntError>` produced by `TryFrom` or `TryInto`.

The lint MUST cover these fallback forms:

- `.unwrap_or(0)`
- `.unwrap_or(<int>::MIN)`
- `.unwrap_or(<int>::MAX)`
- `.unwrap_or_default()`

The lint MUST prefer resolved type information over textual matching so that aliases and qualified `TryFrom`/`TryInto` paths are covered when they produce `TryFromIntError`.

The lint MUST NOT warn when the fallback is propagated, handled through `match`, or converted into a domain-specific error value.

#### Scenario: Zero fallback after narrowing [r[fallible-int-fallback-detection.scenario.zero-fallback]]

- GIVEN production code that evaluates `u32::try_from(count).unwrap_or(0)`
- WHEN the lint runs
- THEN `fallible_int_fallback` warns that the failed conversion is hidden behind a zero fallback

#### Scenario: Extreme fallback after narrowing [r[fallible-int-fallback-detection.scenario.extreme-fallback]]

- GIVEN production code that evaluates `u32::try_from(count).unwrap_or(u32::MAX)` or `i32::try_from(count).unwrap_or(i32::MIN)`
- WHEN the lint runs
- THEN `fallible_int_fallback` warns that the failed conversion is hidden behind an integer extreme fallback

#### Scenario: Default fallback after narrowing [r[fallible-int-fallback-detection.scenario.default-fallback]]

- GIVEN production code that evaluates `u32::try_from(count).unwrap_or_default()`
- WHEN the lint runs
- THEN `fallible_int_fallback` warns that the failed conversion is hidden behind the integer default

#### Scenario: Explicit error handling [r[fallible-int-fallback-detection.scenario.explicit-error-handling]]

- GIVEN production code that matches on `u32::try_from(count)` and returns a domain error on failure
- WHEN the lint runs
- THEN `fallible_int_fallback` does NOT fire
