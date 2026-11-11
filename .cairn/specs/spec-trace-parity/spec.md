# Spec Trace Parity Specification

## Purpose

Defines the `spec-trace-parity` capability.

## Requirements

### Requirement: Spec-trace parity is a versioned Octet check

r[octet.spec_trace_parity.rail]

Octet MUST provide a versioned `spec-trace-parity` check that consumes a pinned model identity, a checker identity and version, a seed, a trace-set identity, and a declared replay-adapter contract, and MUST report parity for the admitted inputs.

#### Scenario: Check is invoked with complete inputs
- GIVEN a crate, a pinned model and checker identity, a seed, a trace set, and a declared adapter
- WHEN the check runs
- THEN it MUST replay every trace in the set and report one deterministic parity verdict.

#### Scenario: Check is invoked without an adapter
- GIVEN a crate and traces but no declared replay adapter
- WHEN the check runs
- THEN it MUST report a deterministic unsupported-verifier result and MUST NOT emit a parity claim.

### Requirement: Freshness binds every layer before replay

r[octet.spec_trace_parity.freshness]

Octet MUST deny a spec-trace-parity run before any replay step when the model hash, checker identity and version, seed, or trace-set identity does not match the declared admission pins.

#### Scenario: All pins are fresh
- GIVEN every declared pin matches the supplied artifact identities
- WHEN freshness admission runs
- THEN replay MAY proceed.

#### Scenario: A pin is stale or mismatched
- GIVEN the checker version, model hash, seed, or trace-set hash is stale or mismatched
- WHEN freshness admission runs
- THEN Octet MUST fail closed and MUST bind the stale pin in the denial receipt.

### Requirement: Replay is seed-bound and deterministic

r[octet.spec_trace_parity.replay]

Octet MUST execute trace replay against the injected adapter with an explicit seed, explicit step and time bounds, and MUST produce identical step sequences and receipts for identical inputs and seed.

#### Scenario: Identical inputs produce identical replay
- GIVEN the same adapter, traces, and seed
- WHEN replay runs twice
- THEN both runs MUST produce the same step sequence and the same receipt identity.

#### Scenario: Replay exceeds bounds
- GIVEN replay exceeds the declared step or time budget
- WHEN the run is bounded
- THEN Octet MUST terminate the run and MUST record a bounded denial rather than an unbounded hang.

### Requirement: Adapters are injected and declare their projection

r[octet.spec_trace_parity.adapter]

Each target crate MUST supply its replay adapter through the Octet adapter surface, and the adapter MUST declare its state projection and any loss bound before replay.

#### Scenario: Adapter declares a projection
- GIVEN an adapter declares a state projection and a loss bound
- WHEN replay is planned
- THEN the plan MUST record the projection and loss bound in the receipt.

#### Scenario: Adapter projection hides a divergence
- GIVEN an adapter maps distinct implementation states onto one model state
- WHEN a divergence would be lost by projection
- THEN the replay record MUST surface the projection as evidence, not as undisclosed parity.

### Requirement: Receipts classify the result exactly

r[octet.spec_trace_parity.receipt]

Octet MUST emit a deterministic receipt binding model hash, checker identity and version, seed, trace-set hash, adapter identity, bounds, verdict class, and divergence samples, and MUST use only the parity, divergence, and unsupported-verifier classes.

#### Scenario: Parity receipt validates
- GIVEN a complete run and receipt
- WHEN receipt validation runs
- THEN every bound identity MUST match the admitted artifacts and the class MUST equal the observed outcome.

#### Scenario: Receipt class is swapped
- GIVEN a receipt claims parity while the observed outcome is divergence
- WHEN receipt validation runs
- THEN Octet MUST fail closed.

### Requirement: Suite has positive and negative fixtures

r[octet.spec_trace_parity.fixtures]

The spec-trace-parity suite MUST include positive fixtures for fresh parity and deterministic replay, and negative fixtures for stale model, stale checker, seed drift, tampered traces, divergence, adapter loss, bound denial, and unsupported verifier.

#### Scenario: Suite runs
- GIVEN all fixtures are committed with their declared identities
- WHEN the validation harness executes
- THEN positive fixtures MUST pass and each negative fixture MUST fail at its declared boundary.

### Requirement: Parity changes run full validation

r[octet.spec_trace_parity.validation]

Changes to the spec-trace-parity check, freshness admission, replay executor, receipts, or fixtures MUST run positive and negative tests, clippy with warnings denied, the full Octet check, lifecycle validation, Cairn gates, and relevant flake checks.

#### Scenario: Change is validated
- GIVEN a spec-trace-parity change passes all focused checks
- WHEN lifecycle validation is assembled
- THEN it MUST include both positive and negative results and MUST identify the exact model, checker, trace, and adapter identities.
