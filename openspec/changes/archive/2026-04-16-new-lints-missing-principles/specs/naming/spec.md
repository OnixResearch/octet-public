## ADDED Requirements

### Requirement: Negated Predicate Detection

The lint MUST warn on:
- boolean-returning functions
- bool-typed local bindings

when their names begin with one of these whole-token negative prefixes:
- `is_not_`
- `has_no_`
- `has_not_`
- `lacks_`
- `cannot_`
- `cant_`
- `does_not_`
- `doesnt_`
- `isnt_`
- `wasnt_`
- `wont_`
- `no_`

The lint MUST suggest a positive reformulation.

The lint MUST use word-boundary semantics for prefix matching, so near-miss
names like `notify` or `notion_ready` do NOT count as negative prefixes.

The `no_` prefix is intentionally broad for bool names. A bool helper like
`no_capacity` still counts as a negated predicate; only infix or near-miss
spellings stay quiet.

The lint MUST NOT fire on non-boolean bindings or functions.

The lint MUST NOT fire on a private helper function named exactly `not`.

#### Scenario: Negated function name

- GIVEN `fn is_not_ready(state: &State) -> bool`
- WHEN the lint runs
- THEN it warns to prefer a positive form like `is_ready`

#### Scenario: Positive function name

- GIVEN `fn is_ready(state: &State) -> bool`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Near-miss identifier

- GIVEN `fn notify(state: &State) -> bool`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Non-boolean variable

- GIVEN `let no_progress: usize = retries_left;`
- WHEN the lint runs
- THEN it does NOT fire
