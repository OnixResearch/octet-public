# Design: Kani panic-freedom pilots

## Approach

The pilot work starts with small, reviewable harnesses over existing functional-core surfaces. Each harness uses nondeterministic inputs, explicit reviewed assumptions, and assertions that map to a named requirement or policy property.

## Candidate surfaces

Preferred pilot surfaces are:

- pure deterministic cores where symbolic inputs avoid mocks;
- arithmetic, normalization, and encode/decode boundaries;
- unsafe or FFI-adjacent wrappers with narrow preconditions;
- state transitions with compact bounded domains;
- code where fuzzing has weak gradients near sparse boundaries.

## Harness convention

Each pilot harness declares its property class, input admission boundary, assumptions, expected Kani mode, and replay path. Assumptions are reviewed like code because vacuous assumptions can make a proof meaningless.

## Positive and negative evidence

Positive evidence records passing panic-freedom or safety assertions. Negative evidence includes a deliberately malformed fixture or counterexample replay path so reviewers can see the harness rejects the wrong behavior.

## What is explicitly not provided

- No default CI requirement for all consumers.
- No replacement for existing Octet lints, property tests, Miri, or Verus rails.
- No unbounded functional claim unless a harness has passing contracts or passing unwinding assertions for that scope.
