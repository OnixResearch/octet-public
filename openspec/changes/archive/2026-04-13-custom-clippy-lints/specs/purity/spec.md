# Purity Lints — Delta Spec

## ADDED Requirements

### Requirement: Verified Module Purity

The lint MUST detect the following in files under any `verified/` directory:

1. `async fn` or `async move` blocks
2. `tokio::` or `async_std::` path references
3. `std::io::` usage (Read, Write, BufReader, etc.)
4. `std::time::SystemTime` or `std::time::Instant` calls
5. `std::fs::` filesystem operations
6. `std::net::` networking
7. `std::process::` subprocess spawning

The lint MUST emit a clear message: "I/O operation in verified module —
verified/ functions must be pure (no async, no IO, no system time)."

The lint MUST NOT fire on `std::time::Duration` (it's a value type, not
a clock read).

The lint MUST NOT fire on `#[cfg(test)]` blocks inside verified modules
(tests may use IO for setup).

The lint SHOULD detect `rand::` usage and suggest passing randomness as
an explicit parameter.

#### Scenario: Async function in verified module

- GIVEN `src/verified/lock.rs` contains `pub async fn check_expiry()`
- WHEN the lint runs
- THEN it warns: "async fn in verified module — verified functions must be synchronous and pure"

#### Scenario: System time in verified module

- GIVEN `src/verified/deadline.rs` calls `SystemTime::now()`
- WHEN the lint runs
- THEN it warns and suggests passing time as a `u64` parameter

#### Scenario: Duration type usage

- GIVEN `src/verified/timeout.rs` uses `Duration::from_millis(500)`
- WHEN the lint runs
- THEN it does NOT fire (Duration is a pure value)

#### Scenario: Test helper in verified module

- GIVEN `src/verified/lock.rs` has `#[cfg(test)] mod tests { use std::fs; }`
- WHEN the lint runs
- THEN it does NOT fire on the test module

### Requirement: Ambient Clock Boundary Ownership

The lint MUST warn on direct wall-clock reads such as `Instant::now()`,
`SystemTime::now()`, `Utc::now()`, or `Local::now()` in production code unless
those reads are isolated behind an explicit shell-boundary helper or an item-
scoped allow with a justification.

The lint MUST suggest injecting time as a parameter for pure/core logic, or
adding a narrow `#[allow(ambient_clock, reason = "...")]` on the boundary
function that legitimately owns time capture.

The lint MUST NOT fire in `#[test]` code or other test-only contexts.

The lint MUST NOT fire on pure duration/value construction such as
`Duration::from_millis(5)`.

#### Scenario: Direct clock read in production logic

- GIVEN `fn retry_deadline() -> Instant { Instant::now() + backoff }`
- WHEN the lint runs
- THEN it warns that the function should receive time as a parameter or isolate
  the clock read at a boundary helper

#### Scenario: Explicit shell boundary helper

- GIVEN a boundary helper marked `#[allow(ambient_clock, reason = "entry point owns time capture")]`
- WHEN the lint runs
- THEN it does NOT fire on that helper

#### Scenario: Test-only clock read

- GIVEN a unit test that calls `Instant::now()` to compare elapsed time
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Contradictory Time Source

The lint MUST warn on functions that both accept a time parameter (name
matching `*_ms`, `*_secs`, `*_us`, `*_ns`, `now`, `timestamp`,
`current_time`, `elapsed`) AND call `SystemTime::now()`, `Instant::now()`,
or `std::time::SystemTime::now()` in their body.

The lint MUST suggest removing the system time call and using the
parameter instead, or removing the parameter if the function should
own its time source.

This lint exists because every time-dependent function in verified-logic
(`expiration.rs`, `rate_limit.rs`, `lease.rs`, `backoff.rs`) takes time
as an explicit parameter — this is what makes them pure, testable, and
verifiable. A function that takes `now_ms` but also calls `Instant::now()`
has two conflicting time sources.

#### Scenario: Parameter and system call

- GIVEN `fn check(now_ms: u64) -> bool { let t = Instant::now(); ... }`
- WHEN the lint runs
- THEN it warns: "function takes `now_ms` parameter but also reads system time"

#### Scenario: Pure time function

- GIVEN `fn is_expired(deadline_ms: u64, now_ms: u64) -> bool { now_ms > deadline_ms }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Shell function with no time param

- GIVEN `fn poll() { let now = Instant::now(); ... }`
- WHEN the lint runs
- THEN it does NOT fire (no conflicting time parameter)
