## ADDED Requirements

### Requirement: Functional lint analysis core

r[octet.lint_analysis_cores.functional_core] Non-trivial lint classification logic MUST be split into a pure analysis core that receives normalized facts and returns a deterministic classification, plus a thin rustc visitor shell that gathers facts and emits diagnostics.

#### Scenario: Pure classifier can be tested without rustc traversal

r[octet.lint_analysis_cores.functional_core.unit]

- GIVEN a target lint has non-trivial classification logic
- WHEN maintainers add or harden that logic
- THEN the lint MUST expose pure helper coverage for positive and negative classifications without requiring a UI fixture for every logical branch.

#### Scenario: Rustc shell remains thin

r[octet.lint_analysis_cores.functional_core.shell]

- GIVEN a lint visitor walks HIR or typed expressions
- WHEN it reaches a candidate source shape
- THEN the visitor SHOULD gather facts, call the pure classifier, and emit through `diagnostics.rs` without embedding unrelated decision logic in the traversal.

### Requirement: Snippet fallback boundaries

r[octet.lint_analysis_cores.snippet_fallback] Source-snippet classification MUST be a documented fallback rather than the primary semantic path when HIR or type facts are available.

#### Scenario: Snippet fallback is documented and guarded

r[octet.lint_analysis_cores.snippet_fallback.guarded]

- GIVEN a lint uses `span_to_snippet` or raw source matching for classification
- WHEN the fallback remains after hardening
- THEN the source file MUST name why structured facts are insufficient.
- AND UI guardrails MUST prove formatting or unrelated comments do not create false diagnostics.

### Requirement: Target lint hardening

r[octet.lint_analysis_cores.public_unsafe_api] `public_unsafe_api` MUST classify unsafe public APIs from HIR safety, visibility, trait, and ABI facts instead of relying on source substrings.

#### Scenario: Extern and unsafe are distinguished structurally

r[octet.lint_analysis_cores.public_unsafe_api.extern]

- GIVEN a public item is `extern` but not a public Rust `unsafe fn` API requiring caller obligations
- WHEN `public_unsafe_api` runs
- THEN the lint MUST not rely on substring matching to decide the diagnostic.

### Requirement: Multi-lock ordering typed facts

r[octet.lint_analysis_cores.multi_lock_ordering] `multi_lock_ordering` SHOULD resolve lock field types structurally and SHOULD require an order-specific comment rather than any comment containing a loose ordering word.

#### Scenario: Unrelated ordering comment does not suppress

r[octet.lint_analysis_cores.multi_lock_ordering.comment]

- GIVEN a struct has multiple lock fields and a nearby comment using `ordering` for an unrelated concept
- WHEN `multi_lock_ordering` runs
- THEN the lint SHOULD still warn unless the comment documents lock acquisition order.

### Requirement: Structural loop-bound analysis

r[octet.lint_analysis_cores.unbounded_loop] `unbounded_loop` SHOULD identify explicit loop bounds, cancellation paths, and structural exits before considering source-marker fallbacks.

#### Scenario: Named marker alone is insufficient

r[octet.lint_analysis_cores.unbounded_loop.marker]

- GIVEN a loop body contains a variable name such as `attempt` without a structural bound or exit
- WHEN `unbounded_loop` runs
- THEN the name alone SHOULD NOT suppress the diagnostic.

### Requirement: Magic number context analysis

r[octet.lint_analysis_cores.magic_number] `magic_number` SHOULD classify policy thresholds from expression context and named bindings rather than only a fixed domain-word list.

#### Scenario: Documented local threshold is clean

r[octet.lint_analysis_cores.magic_number.threshold]

- GIVEN a numeric literal is assigned to a named constant or configuration field that documents the policy threshold
- WHEN `magic_number` runs
- THEN the lint SHOULD emit zero diagnostics for that literal.
