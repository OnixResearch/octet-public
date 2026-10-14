# Benchmark Evidence Specification

## Purpose

Defines the `benchmark-evidence` capability.

## Requirements

### Requirement: Benchmark suites use typed bounded profiles

r[octet.benchmark_evidence.profile]

Octet MUST run benchmarks only through validated Nickel profiles that bind workload, command, mode, cache state, toolchain, environment, sample plan, metrics, limits, comparison policy, and non-claims.

#### Scenario: Reviewed profile constructs a suite

- GIVEN a complete profile selects registered workloads, fixed command IDs, modes, environments, sample plans, metrics, limits, and non-claims
- WHEN benchmark preflight executes
- THEN Octet MUST construct every command and preparation step deterministically
- AND the suite identity MUST bind every behavior-affecting profile field

#### Scenario: Arbitrary benchmark authority fails

- GIVEN a profile includes free-form command text, an unmeasured mutable workload, ambient environment input, unsafe paths, or network-dependent preparation
- WHEN preflight executes
- THEN Octet MUST reject the suite before workload preparation or process execution

### Requirement: Performance protocol preserves raw bounded observations

r[octet.benchmark_evidence.performance_protocol]

Each performance run MUST preserve bounded raw samples, command outcomes, workload identity, cache mode, source state, and environment facts before summary calculation.

#### Scenario: Performance sample set completes

- GIVEN every declared sample exits successfully under the selected workload, mode, environment, and limits
- WHEN Octet records the sample set
- THEN the receipt MUST preserve each wall-duration observation and declared CPU, memory, output, or cache observation
- AND cold, warm, no-change, and incremental modes MUST remain separate result groups

#### Scenario: Sample is invalid

- GIVEN a sample fails, times out, exceeds limits, uses a changed workload, or lacks a required observation
- WHEN sample validation runs
- THEN the sample MUST remain visible with its failure class
- AND it MUST NOT be silently discarded from a passing summary

### Requirement: Signal benchmarks use an independent oracle

r[octet.benchmark_evidence.signal_oracle]

Signal benchmarks MUST compare Octet findings with an adjudicated oracle that binds expected lint identity, location class, outcome, rationale, reviewer role, source identity, and unresolved cases.

#### Scenario: Findings align with the oracle

- GIVEN a workload and oracle have matching source identities
- WHEN the pure alignment core evaluates Octet findings
- THEN it MUST classify true positives, false positives, false negatives, expected suppressions, unexpected suppressions, and inconclusive cases deterministically
- AND Octet output MUST NOT modify the oracle during evaluation

#### Scenario: Oracle is stale or contradictory

- GIVEN an oracle references stale source, duplicate expected findings, contradictory outcomes, unknown lint identities, or missing reviewer rationale
- WHEN oracle validation runs
- THEN validation MUST fail before signal metrics are calculated

### Requirement: Benchmark comparisons require compatible evidence

r[octet.benchmark_evidence.comparison]

Octet MUST compare baseline and candidate results only when required workload, profile, command, mode, cache, toolchain, environment, sample, metric, and oracle identities are compatible.

#### Scenario: Compatible results compare

- GIVEN baseline and candidate receipts satisfy all declared comparability rules
- WHEN the pure comparison core runs
- THEN it MUST report `improved`, `stable`, `regressed`, or `inconclusive` from retained samples and declared policy
- AND the result MUST bind the practical threshold and uncertainty rule

#### Scenario: Required identity differs

- GIVEN one required comparability identity differs or is unavailable
- WHEN comparison runs
- THEN the result MUST be `incomparable`
- AND Octet MUST NOT report improvement, stability, or regression

#### Scenario: Single noisy sample is supplied

- GIVEN a profile requires repeated observations but a result contains one sample or an invalid sample set
- WHEN comparison runs
- THEN the result MUST fail validation or remain `inconclusive`
- AND it MUST NOT become a performance gate

### Requirement: Benchmark receipts are independently verifiable

r[octet.benchmark_evidence.receipt]

A benchmark receipt MUST bind profiles, workloads, commands, source states, environments, raw samples, signal oracles, findings, summaries, comparisons, logs, limits, parent links, and BLAKE3 identities.

#### Scenario: Receipt reproduces its summary

- GIVEN a complete benchmark artifact bundle
- WHEN artifact validation runs
- THEN validation MUST remeasure every parent and raw artifact
- AND it MUST reproduce summary and comparison results from bound inputs and policy

#### Scenario: Raw artifact is stale

- GIVEN a workload, profile, sample, oracle, finding set, summary, comparison, or parent changes after collection
- WHEN artifact validation runs
- THEN validation MUST fail with a deterministic identity diagnostic

### Requirement: Benchmark evidence remains bounded

r[octet.benchmark_evidence.boundary]

Benchmark receipts MUST remain optional `transcript_assertion` evidence and MUST state workload, environment, corpus, noise, measurement, and representativeness boundaries.

#### Scenario: Selected corpus is reported

- GIVEN a benchmark suite covers synthetic fixtures or selected projects
- WHEN Octet renders the receipt
- THEN it MUST identify the exact corpus and unresolved cases
- AND it MUST NOT claim representation of all Rust projects, workloads, defects, or environments

#### Scenario: Starter checks run

- GIVEN no benchmark profile is selected
- WHEN starter `cargo octet check` runs
- THEN benchmark workloads, tools, artifacts, and environments MUST NOT be required

### Requirement: Public benchmark claims cite verified evidence

r[octet.benchmark_evidence.public_claims]

Every published Octet performance, cache, memory, signal, suppression, false-positive, or false-negative metric MUST cite a verified receipt and its interpretation boundary.

#### Scenario: Public metric is reviewable

- GIVEN Octet documentation publishes a benchmark value or comparison
- WHEN a reviewer follows its evidence reference
- THEN the reference MUST identify the verified receipt, profile, corpus, environment, sample range, result, and non-claims

#### Scenario: Unsupported headline metric fails review

- GIVEN documentation publishes a benchmark claim without verified evidence or extends it beyond the measured corpus and environment
- WHEN benchmark claim validation runs
- THEN validation MUST fail with a missing-evidence or overclaim diagnostic

### Requirement: Benchmark verification covers positive and negative paths

r[octet.benchmark_evidence.verification]

The implementation MUST include deterministic positive and negative fixtures for profiles, workloads, samples, environments, signal oracles, finding alignment, summaries, comparisons, identities, limits, compatibility, and claims.

#### Scenario: Focused benchmark matrix completes

- GIVEN stable, improved, regressed, inconclusive, incomparable, warm, cold, incremental, signal, stale, malformed, failed, over-limit, and overclaim fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
