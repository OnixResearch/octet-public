## Context

`lint_emitting_fixtures.json` covers positive diagnostics. A companion negative manifest should prove each lint also has a clean representative case, disabled/configured behavior where relevant, and at least one known false-positive guardrail for heuristic lints.

## Decisions

### 1. Add a separate non-emitting manifest

**Choice:** Introduce `tests/fixtures/lint_guardrail_fixtures.json` mapping each lint to one or more non-emitting fixtures with a scenario label.

**Rationale:** Positive and negative coverage have different acceptance criteria. Keeping them separate avoids overloading existing emitting coverage.

### 2. Enforce completeness in library tests

**Choice:** Add a pure manifest comparison that checks every inventory lint has emitting and guardrail coverage, and that every manifest fixture exists in `ui_fixture_set.json`.

**Rationale:** New lints should fail fast if they skip negative coverage.

### 3. Support lint-specific boundary categories

**Choice:** The manifest records categories such as `safe_shape`, `configured_off`, `within_limit`, `test_context`, `macro_guardrail`, or `known_exception`.

**Rationale:** A single clean fixture is useful, but reviewers need to know which failure mode it protects.

## Risks / Trade-offs

- Requiring guardrails may add fixture churn. This is intentional; each lint should document its non-warning boundary.
- Some lints share fixture files. The manifest may point multiple lints at one fixture when the stderr remains clean for all relevant lint names.

## Validation

Run `cargo test --lib` for manifest freshness and the ignored UI rail to prove guardrail fixtures compile without unexpected diagnostics.
