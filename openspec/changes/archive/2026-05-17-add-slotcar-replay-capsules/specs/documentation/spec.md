## ADDED Requirements

### Requirement: Replay Capsule Operator Guidance [r[replay-capsule-operator-guidance]]

The system MUST document how operators capture, inspect, and replay Slotcar replay capsules for lint and fingerprint evidence. The guidance MUST name exact commands, required inputs, redaction guarantees, and evidence paths suitable for OpenSpec review.

#### Scenario: Operator guide names capture and replay commands [r[replay-capsule-operator-guidance.scenario.commands-named]]

- GIVEN an operator needs to preserve a release, review, or bug-reproduction Slotcar run
- WHEN they read the replay-capsule documentation
- THEN the guide names the capture, inspect, and replay commands
- AND it states which capsule fields are required for `cargo slotcar check` and `cargo slotcar fingerprint` evidence

#### Scenario: Operator guide states replay limits [r[replay-capsule-operator-guidance.scenario.limits-stated]]

- GIVEN a replay capsule reports `match`
- WHEN an operator reads the guidance
- THEN the guidance states that the capsule proves reproducible evidence only
- AND it does not claim semantic correctness, safety, or verification solely from matching hashes
