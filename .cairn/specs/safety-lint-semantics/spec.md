# Safety Lint Semantics Specification

## Purpose

Defines the `safety-lint-semantics` capability.

## Requirements

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

#### Scenario: Async fn wildcard parameter is not a discard

r[octet.safety_lint_semantics.discards.async_parameter]

- GIVEN `async fn on_done(&self, _: io::Result<()>)`, which rustc lowers into a `let _ = ...` statement inside the async body
- WHEN `ignored_result` or `ignored_join_handle` runs
- THEN it MUST emit zero diagnostics for that parameter
- AND `let _ = fallible()` and `_ = fallible()` written in the body MUST still be flagged.

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

A `return` counts as a reachable return path even when an expansion produced it, such as the `return` that `?` lowers to or one inside an `ensure!`-style macro. A `break` produced by an expansion, such as the `while` desugaring's own `break`, does not count as an exit.

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

### Requirement: Declared durability boundary profile
r[octet.durability_claims.boundary_profile] Octet MUST analyze durability claims only within consumer-declared source scopes and exact boundary surfaces.

#### Scenario: No durability boundary is declared
- GIVEN a consumer without a durability-boundary profile
- WHEN Octet checks ordinary `flush` or commit calls
- THEN it does not invent a product durability claim

### Requirement: Synchronization result handling
r[octet.durability_claims.sync_result] Octet MUST report direct discard, `let _ =`, empty error handling, or unconditional error-to-success mapping for declared synchronization and commit results.

#### Scenario: Sync result is discarded
- GIVEN a declared boundary that uses `let _ = file.sync_all()`
- WHEN source analysis runs
- THEN Octet reports the configured durability finding

#### Scenario: Sync result is handled
- GIVEN a declared boundary that maps synchronization failure to explicit uncertainty and quarantine
- WHEN source analysis runs
- THEN this rule does not report result discard

### Requirement: Flush-only durability finding
r[octet.durability_claims.flush_only] `Write::flush()` MUST NOT satisfy a declared durable-persistence observation by itself.

#### Scenario: Declared boundary uses flush only
- GIVEN a declared durable boundary that ends after handled `flush()`
- WHEN source analysis runs
- THEN Octet reports missing durable observation

### Requirement: Retry after synchronization error
r[octet.durability_claims.retry_after_error] Octet MUST report the bounded same-receiver retry shape when no intervening write, reopen, replacement handle, or reconciliation appears in the analyzed block.

#### Scenario: Failed sync is retried directly
- GIVEN a handled sync error followed by another sync on the same visible receiver
- WHEN no admitted recovery operation appears between them
- THEN Octet reports the retry shape

#### Scenario: Failed sync is retried after rewrite
- GIVEN a handled sync error, an explicit rewrite, and a later handled sync
- WHEN source analysis runs
- THEN the direct-retry rule does not report that shape

### Requirement: Durability finding guidance
r[octet.durability_claims.guidance] Diagnostics MUST explain explicit uncertain outcomes, quarantine, reopen, rewrite, and reconciliation without claiming one universal repair.

#### Scenario: Finding is rendered
- GIVEN one durability finding
- WHEN Octet renders guidance
- THEN it states the source-shape scope and durability non-claims

### Requirement: Durability lint validation
r[octet.durability_claims.validation] Every new durability rule MUST include positive and negative UI, policy, and evidence fixtures.

#### Scenario: Invalid profile is supplied
- GIVEN a durability profile with an unknown observation class
- WHEN policy validation runs
- THEN validation fails closed
