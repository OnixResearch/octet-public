# Structure Skill Coverage — Delta Spec

## ADDED Requirements

### Requirement: Function Interface And Data-Handling Coverage In Skill

The Tiger Style skill MUST include the missing structure guidance for
function design and data handling.

The skill MUST cover:

- simplifying function signatures
- constructing large objects in place
- passing large objects by reference
- minimizing dimensionality in parameters and return types
- handling buffer allocation and cleanup in one logical block
- handling division intentionally with explicit rounding intent
- minimizing variable scope

#### Scenario: Reviewing a large by-value parameter

- GIVEN a function takes a large struct by value without ownership need
- WHEN an agent follows the skill
- THEN it is prompted to prefer passing by reference

### Requirement: Formatting And Documentation Norms In Skill

The Tiger Style skill MUST include the missing code-consistency and
communication rules from the reference.

The skill MUST cover:

- consistent indentation
- reasonable line-length limits
- clear code block separation
- consistent bracing rules
- standardized tooling where practical
- standalone commit messages that include the why
- documentation of test methodology, including how a test validates a property

#### Scenario: Writing a test comment

- GIVEN a correctness test for mutual exclusion
- WHEN an agent follows the skill
- THEN it documents how the test establishes the property, not only the property name
