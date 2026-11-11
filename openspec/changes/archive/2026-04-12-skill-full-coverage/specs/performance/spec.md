# Performance Guidance — Delta Spec

## ADDED Requirements

### Requirement: Full Performance Principle Coverage In Skill

The Tiger Style skill MUST cover the full performance guidance from the
reference, not only the high-level themes.

The skill MUST include:

- design for performance early
- napkin math
- analysis of bandwidth AND latency
- separation of control plane and data plane
- batching
- resource priority ordering
- optimization by cost × frequency
- large CPU work chunks
- hot-loop extraction with primitive arguments
- predictability for cache and branch behavior
- reduced dependence on compiler optimization luck

#### Scenario: Reviewing a hot path

- GIVEN an agent is reviewing a performance-sensitive path
- WHEN it follows the skill
- THEN it is prompted to reason about latency, bandwidth, frequency, batching, and predictability rather than only micro-optimizations

### Requirement: Napkin Math Example Or Equivalent Guidance

The skill MUST include either a worked napkin-math example or an explicit,
nearby pointer to the worked example in the reference.

The skill MUST explain that napkin math is used for early bottleneck
identification, sanity checks, and rough cost estimation.

#### Scenario: Estimating storage cost

- GIVEN an agent needs to evaluate a logging design
- WHEN it reads the skill
- THEN it can discover that rough throughput and storage calculations are part of Tiger Style, not optional extra analysis
