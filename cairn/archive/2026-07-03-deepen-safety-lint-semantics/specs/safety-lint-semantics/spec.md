## ADDED Requirements

### Requirement: Broadened discard semantics

r[octet.safety_lint_semantics.discards] Octet safety lints MUST detect high-signal typed discard shapes for fallible and must-use values without regressing documented shutdown, drop, and test-context exceptions.

#### Scenario: Statement-position Result discard is flagged

r[octet.safety_lint_semantics.discards.statement]

- GIVEN production code evaluates a `Result` in statement position without `?`, `match`, logging, or explicit handling
- WHEN discard semantics run
- THEN Octet SHOULD emit an `ignored_result` diagnostic or an equivalent safety-family diagnostic.

#### Scenario: Documented shutdown send remains clean

r[octet.safety_lint_semantics.discards.shutdown]

- GIVEN a shutdown or close path intentionally ignores a supported sender result under the documented exception
- WHEN `ignored_result` runs
- THEN it MUST continue to emit zero diagnostics for that exception.

### Requirement: Recursion semantics

r[octet.safety_lint_semantics.recursion] `no_recursion` MUST detect direct self-recursion through functions and methods, and MAY add advisory mutual-recursion detection only with fixture-backed low-noise evidence.

#### Scenario: Method self-recursion is flagged

r[octet.safety_lint_semantics.recursion.method]

- GIVEN an inherent method calls itself through the current receiver or its resolved method DefId
- WHEN `no_recursion` runs
- THEN it MUST emit a diagnostic naming the recursive call site.

#### Scenario: Non-recursive trait dispatch is clean

r[octet.safety_lint_semantics.recursion.trait_guardrail]

- GIVEN a trait method implementation calls a different required trait method or a supertrait helper
- WHEN `no_recursion` runs
- THEN it MUST NOT warn unless the resolved call is actually recursive.

### Requirement: Structural loop-bound semantics

r[octet.safety_lint_semantics.loop_bounds] `unbounded_loop` MUST prefer structural bounds, exits, cancellation, and bounded receiver facts over source-name marker heuristics.

#### Scenario: Bounded retry loop is clean

r[octet.safety_lint_semantics.loop_bounds.retry]

- GIVEN a retry loop has an explicit maximum attempt counter and reachable break or return path
- WHEN `unbounded_loop` runs
- THEN it SHOULD emit zero diagnostics.

#### Scenario: Runaway loop remains flagged despite marker names

r[octet.safety_lint_semantics.loop_bounds.runaway]

- GIVEN a `loop` contains variable names such as `attempt` or `remaining` but no structural exit or bound
- WHEN `unbounded_loop` runs
- THEN it MUST emit the unbounded-loop diagnostic.
