# Concurrency Lints — Delta Spec

## ADDED Requirements

### Requirement: Multiple Locks Without Documented Ordering

The lint MUST be able to warn on struct definitions containing 2 or more fields
of type `Mutex<_>`, `RwLock<_>`, `tokio::sync::Mutex<_>`, or
`tokio::sync::RwLock<_>` when there is no comment in the struct body
documenting lock acquisition order.

The lint MUST accept a comment containing "lock order" or "ordering"
anywhere in the struct definition.

This lint exists because clankers has structs like `LivenessTracker`
with `last_output: Mutex<Instant>` and `state: Mutex<HealthState>`,
and `CredentialManager` with `credential: Mutex<Credential>` and
`refresh_guard: Mutex<()>`. Undocumented lock ordering leads to
deadlocks that only appear under production concurrency.

#### Scenario: Two mutexes without ordering comment

- GIVEN:
  ```rust
  struct Tracker {
      count: Mutex<u64>,
      state: Mutex<State>,
  }
  ```
- WHEN the lint runs
- THEN it warns: "struct has 2 Mutex fields without documented lock ordering"

#### Scenario: Two mutexes with ordering comment

- GIVEN:
  ```rust
  struct Tracker {
      // Lock ordering: always acquire `count` before `state`
      count: Mutex<u64>,
      state: Mutex<State>,
  }
  ```
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Single mutex

- GIVEN a struct with exactly one `Mutex` field
- WHEN the lint runs
- THEN it does NOT fire (no ordering concern)
