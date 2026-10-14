## ADDED Requirements

### Requirement: Sentinel Fallback Detection

The lint MUST default to `warn`.

The lint MUST warn when code uses `Option::unwrap_or`,
`Option::unwrap_or_else`, `Result::unwrap_or`, or `Result::unwrap_or_else` to
replace absence or failure with an integer extreme sentinel for the same target
integer type.

For the first rollout, the accepted sentinel forms are `<int>::MAX` and
`<int>::MIN`, where `<int>` is one of `u8..u128`, `i8..i128`, `usize`, or
`isize`.

For this rule, `same target integer type` means the sentinel constant's type
matches the type returned by the `unwrap_or` / `unwrap_or_else` call.

The lint MUST use receiver type and resolved constant identity rather than raw
source-text matching.

The lint MUST explain that the sentinel masks absence or failure and MUST
recommend an explicit branch, propagated error/option state, or a domain-tagged
fallback representation.

The lint MUST fire at the `unwrap_or` / `unwrap_or_else` call site regardless of
how the resulting value is later used, including when that value is immediately
formatted into a string, key, or identifier.

The lint MUST NOT fire when code branches explicitly on the `Option` or
`Result` instead of forcing an extreme sentinel.

The lint MUST NOT fire on non-extreme defaults such as `0`, `1`, or other
non-`MIN` / `MAX` fallback expressions.

The lint MUST continue to respect normal rustc allow / deny handling,
including `#[allow(sentinel_fallback)]`.

The lint MUST NOT fire in the repo's supported test contexts.

#### Scenario: Fallible conversion masked by max sentinel

- GIVEN `format!("recent_{}", u64::try_from(command_index).unwrap_or(u64::MAX))`
- WHEN the lint runs
- THEN it warns that the code masks conversion failure with an extreme sentinel

#### Scenario: Optional index masked by max sentinel

- GIVEN `let idx = maybe_index.unwrap_or(u32::MAX);`
- WHEN the lint runs
- THEN it warns that the code masks absence with an extreme sentinel

#### Scenario: Side-effect closure masked by min sentinel

- GIVEN `let idx = index_result.unwrap_or_else(|err| { log::warn!("{err}"); i64::MIN });`
- WHEN the lint runs
- THEN it warns that the code still masks absence or failure with an extreme sentinel

#### Scenario: Explicit branch stays quiet

- GIVEN code that uses `match` or `if let` to handle the missing or failing
  case explicitly before formatting or returning a value
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-extreme default stays quiet

- GIVEN `let idx = maybe_index.unwrap_or(0);`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-extreme closure default stays quiet

- GIVEN `let idx = maybe_index.unwrap_or_else(|_| 0);`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Test-context sentinel fallback stays quiet

- GIVEN `unwrap_or(u64::MAX)` appears inside one of the repo's supported test
  contexts
- WHEN the lint runs
- THEN it does NOT fire
