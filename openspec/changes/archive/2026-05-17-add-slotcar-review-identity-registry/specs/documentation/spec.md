## ADDED Requirements

### Requirement: Review Identity Operator Guidance [r[review-identity-operator-guidance]]

The system MUST document how operators create, inspect, update, and roll back Slotcar review identity registry entries. The guidance MUST explain the distinction between mutable labels and immutable target hashes.

#### Scenario: Guidance explains name and hash roles [r[review-identity-operator-guidance.scenario.name-hash-roles]]

- GIVEN an operator reads the review identity guide
- WHEN the guide introduces labels and target hashes
- THEN it states that labels are ergonomic mutable pointers
- AND immutable hashes are the durable evidence identity used for validation and replay

#### Scenario: Guidance documents rollback [r[review-identity-operator-guidance.scenario.rollback]]

- GIVEN an operator needs to revert a review label to a previous evidence target
- WHEN they read the guide
- THEN the guide names the inspect and rollback workflow
- AND it states that rollback changes the pointer only, not the immutable target records
