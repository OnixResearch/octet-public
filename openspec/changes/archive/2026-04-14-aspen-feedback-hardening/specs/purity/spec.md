# Purity — Delta Spec

## ADDED Requirements

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
