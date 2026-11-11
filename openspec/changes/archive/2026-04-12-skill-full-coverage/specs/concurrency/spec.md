# Concurrency Skill Coverage — Delta Spec

## ADDED Requirements

### Requirement: Unsafe Quarantine Guidance In Skill

The Tiger Style skill MUST include the reference guidance that
safety-critical concurrency internals may use `unsafe`, but public APIs
must remain safe, small, and auditable.

The skill MUST distinguish between:

- acceptable localized internal unsafety behind a safe wrapper, and
- unsafe escape hatches that must be explicit, grep-able, and dangerous by name

#### Scenario: Reviewing a lock primitive

- GIVEN a concurrency primitive with internal raw synchronization
- WHEN an agent follows the skill
- THEN it expects the public API to stay safe and the unsafety to be localized to a narrow implementation boundary
