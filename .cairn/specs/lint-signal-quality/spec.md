# Lint Signal Quality Specification

## Purpose

Defines the `lint-signal-quality` capability.

## Requirements

### Requirement: Dogfood signal-quality budget

r[octet.lint_signal_quality.dogfood_budget] Octet MUST record signal-quality summaries for high-volume lint families before promoting those lints beyond inventory or advisory rollout.

#### Scenario: Dogfood summary separates raw and residual findings

r[octet.lint_signal_quality.dogfood_budget.summary]

- GIVEN a dogfood run enables high-volume naming or indexable lints
- WHEN the summary is rendered
- THEN it MUST report raw findings, configured exceptions, accepted debt, fixed findings, and residual findings per lint.
- AND it MUST name the command, enabled lint set, disabled lint count, and artifact path for the raw finding inventory.

#### Scenario: Residual noise blocks promotion

r[octet.lint_signal_quality.dogfood_budget.blocking]

- GIVEN a lint retains high accepted-debt or residual-noise counts after scoped exceptions are applied
- WHEN maintainers evaluate promotion beyond inventory mode
- THEN the promotion MUST be blocked or explicitly deferred with a saved triage rationale.

### Requirement: Scoped exception classes

r[octet.lint_signal_quality.scoped_exceptions] Octet MUST prefer typed, scoped exception classes over broad disables for high-volume naming/indexing lints.

#### Scenario: Rustc-private lint implementation aliases are exempted narrowly

r[octet.lint_signal_quality.scoped_exceptions.rustc_alias]

- GIVEN a lint implementation imports a canonical rustc-private alias such as `rustc_hir as hir`
- WHEN the affected naming lint runs with the scoped exception enabled
- THEN the alias SHOULD be classified as an accepted implementation shorthand instead of an actionable warning.
- AND the same lint MUST continue warning on unrelated consumer aliases that hide searchable owner paths.

#### Scenario: Lint-identity-preserving names stay searchable

r[octet.lint_signal_quality.scoped_exceptions.identity_names]

- GIVEN a module, type, or function intentionally repeats a lint identity to preserve traceability between source, fixture, and diagnostic metadata
- WHEN `path_segment_repetition` or `underscore_in_module_filename` evaluates that item
- THEN the lint SHOULD suppress or downgrade the finding only for the named traceability class.

### Requirement: Promotion blockers

r[octet.lint_signal_quality.promotion_blockers] Lints with high accepted-debt counts MUST remain inventory-only or advisory until dogfood evidence proves residual findings are actionable.

#### Scenario: Promotion evidence names blocker status

r[octet.lint_signal_quality.promotion_blockers.evidence]

- GIVEN a candidate high-volume lint is considered for starter profile or default-level promotion
- WHEN its promotion evidence is reviewed
- THEN the evidence MUST state `ready`, `deferred`, or `blocked` and cite the latest dogfood summary.

### Requirement: Compiler-owned file diagnostics

r[octet.lint_signal_quality.source_ownership] File-level naming and module-count lints MUST inspect local admitted Rust source files without treating imported dependency files as consumer-owned files.

#### Scenario: Invalid local file remains actionable

- GIVEN a local admitted Rust source file violates the filename rule
- WHEN Octet checks its crate
- THEN the filename finding MUST remain actionable.

#### Scenario: Imported dependency files do not contaminate the consumer

- GIVEN rustc imports a dependency source file with an underscore filename
- AND the dependency shares the consumer workspace directory
- WHEN Octet checks the consumer crate
- THEN the dependency file MUST NOT produce a consumer filename finding or increase the consumer module count.

#### Scenario: Valid local file remains accepted

- GIVEN a local Rust source file satisfies the filename rule
- WHEN Octet checks its crate
- THEN Octet MUST NOT emit a filename finding for that file.
