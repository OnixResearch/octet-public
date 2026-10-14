# Purity Specification

## Purpose

Tiger Style purity requirements for the Functional Core, Imperative Shell
boundary and the associated Verus verification workflow.
## Requirements
### Requirement: Full Verus Workflow Coverage In Skill

The Tiger Style skill MUST include the missing Verus workflow guidance
from the reference.

The skill MUST cover:

- when inline `requires`/`ensures` is enough
- when to split into `spec fn` / `proof fn` files
- the split-directory pattern separating production code and `verus/` specs
- `spec fn`, `exec fn`, `proof fn`, and `#[verifier(external_body)]`
- state-machine invariant proof patterns
- verification command examples
- Verus best practices for invariants, overflow predicates, saturating arithmetic, and time injection

#### Scenario: Complex reusable proof

- GIVEN a proof obligation reused across multiple exec functions
- WHEN an agent follows the skill
- THEN it can identify that the split `verus/` pattern is preferred over crowding the exec file

### Requirement: Functional Core Guidance Includes Verification Context

The skill MUST connect Functional Core / Imperative Shell guidance with
how pure code is tested and verified.

The skill MUST mention that pure core logic is suitable for exhaustive
unit tests, property testing, and formal verification.

#### Scenario: Evaluating a pure helper module

- GIVEN a pure module with no I/O
- WHEN an agent applies the skill
- THEN it recognizes the module as a candidate for property tests and Verus verification guidance

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

### Requirement: Ambient Clock Boundary Pattern Guidance

The project MUST document concrete boundary patterns for `ambient_clock`
findings that appear during downstream rollouts.

The guidance MUST cover:

- timeout-to-deadline conversion helpers
- elapsed-time measurement in worker, retry, or polling loops
- repeated `Instant::now()` checks inside shell-owned wait loops
- how to decide between injecting time as a parameter, centralizing one
  boundary helper, or using a narrow item-scoped `#[allow(ambient_clock,
  reason = "...")]`
- how this guidance relates to `contradictory_time`, so maintainers do not add
  ambient clock reads to functions that already accept time parameters

#### Scenario: Timeout helper in shell code

- GIVEN a downstream crate computes `let deadline = Instant::now() + timeout`
  in a coordination helper
- WHEN a maintainer consults the `ambient_clock` guidance
- THEN they can decide whether that logic belongs in a shared shell-boundary
  helper or should receive `now` as an explicit parameter instead of
  scattering repeated clock reads

#### Scenario: Elapsed-time measurement loop

- GIVEN a worker loop measures elapsed time with `Instant::now()` while owning
  the scheduling boundary
- WHEN a maintainer consults the guidance
- THEN they can see whether a narrow boundary-scoped allow is justified and
  what rationale text to record

#### Scenario: Function already takes time as input

- GIVEN a function already accepts `now`, `elapsed`, or another time parameter
- WHEN the maintainer checks the guidance after an `ambient_clock` warning
- THEN the guidance points them to parameter injection and `contradictory_time`
  instead of approving another direct clock read

