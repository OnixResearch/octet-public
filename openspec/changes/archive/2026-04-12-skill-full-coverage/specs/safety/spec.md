# Safety Skill Coverage — Delta Spec

## ADDED Requirements

### Requirement: Full Assertion And Error-Handling Coverage In Skill

The Tiger Style skill MUST include the missing assertion and
error-handling guidance from the reference.

The skill MUST cover:

- conditional assertions for implications
- assertions on function arguments and return values
- invariant validation during execution
- pair assertions across critical points
- fail-fast handling of programmer errors
- exhaustive testing of valid AND invalid paths
- treating compiler warnings as errors
- avoiding implicit defaults in library calls

#### Scenario: Reviewing a guarded invariant

- GIVEN code where an invariant only holds under a precondition
- WHEN an agent follows the skill
- THEN it prefers `if precondition { assert!(invariant); }` over masking the invariant inside a compound assertion

### Requirement: Positive And Bounded Control Flow Guidance

The Tiger Style skill MUST include the missing safety guidance for
positive predicates and bounded, understandable control flow.

The skill MUST cover:

- minimizing variable scope
- stating invariants positively
- buffering external events before processing
- decomposing compound conditions into one-check branches

#### Scenario: Naming a readiness predicate

- GIVEN a boolean predicate in review
- WHEN an agent applies the skill
- THEN it prefers `is_ready` or `has_quorum` over double-negation style names
