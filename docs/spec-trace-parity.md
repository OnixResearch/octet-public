# Spec-trace parity rail

The spec-trace parity rail replays pinned, seed-bound traces from a behavioral
model against a target crate through an injected adapter. It is a checked
admission surface, not a model checker: Octet owns identity binding, freshness,
bounded replay, verdict classification, and deterministic receipts. The model
layer owns model production and the target crate owns its replay adapter.

The pure core lives in
`octet_standards::spec_trace_parity`. It does not read files, spawn processes,
inspect a clock, or discover adapters.

## Check surface

A run is bound to six identities:

| Identity | Meaning |
| --- | --- |
| `model_identity` | BLAKE3 identity of the exact behavioral model bytes |
| `checker_identity` | BLAKE3 identity of the model checker build |
| `checker_version` | Declared model checker version |
| `seed` | Explicit replay seed |
| `trace_set_identity` | BLAKE3 identity of the canonical trace set |
| `adapter_identity` | BLAKE3 identity of the adapter declaration, or absent |

The trace set must declare the stack interchange identity
`octet.spec-trace/v1`. Foreign formats are mapped to stack identity by Kamacite
before they reach this rail; the rail rejects an unknown format rather than
guessing.

## Freshness

`admit` runs before any replay step and fails closed on a stale model, checker
identity, checker version, seed, trace-set identity, adapter identity, trace
format, or malformed adapter declaration. Each denial binds the denied layer,
the declared value, and the observed value. The denial receipt binds every
declared pin, so a stale pin is evidence rather than a silent skip.

## Adapter contract

Each target crate supplies its adapter through the injected `TraceAdapter`
port. The adapter must declare:

- a stable adapter identifier;
- the trace format it consumes;
- a state projection naming both state vocabularies and its description; and
- a loss bound, either `Exact` or `Bounded { bound }`.

An adapter that omits its projection or loss bound is rejected before replay.
A bounded (lossy) projection may still report parity, but the receipt records
the projection and loss bound and adds the projection non-claim. Parity under a
lossy projection is never reported as undisclosed parity: divergences merged by
the projection are outside the claim.

## Replay

`plan` records the admitted identities, the adapter projection, the loss bound,
and the declared bounds. `run` replays every trace in declared order with the
admitted seed. The core enforces:

- `max_steps`: a trace set that cannot fit the step budget is denied before any
  step runs;
- `max_millis`: the shell reports cumulative elapsed time with each adapter
  observation, and the core stops after the first observation that exceeds the
  budget.

Identical adapters, traces, seeds, and reported timings produce identical step
sequences and receipt identities. Receipts bind the declared bounds, not raw
clock readings, so a time-budget denial is classified deterministically.

## Verdicts and receipts

The closed verdict vocabulary is `parity`, `divergence`, and
`unsupported-verifier`. A run that fails freshness or exceeds a bound is not a
verdict: its receipt records a deterministic denial instead of claiming a
class.

A receipt binds the model, checker, seed, trace-set, adapter, bounds, verdict
class or denial layers, replayed step count, bounded divergence samples, and
non-claims. `validate_receipt` recomputes the admitted identities, the replay
prefix, the divergence samples, and the verdict class from the observed replay
record; a receipt whose class, identities, step count, or samples disagree with
the observed run fails closed.

## Integration boundary

The rail is a pure checked core with an injected adapter port. There is no
`cargo octet` or `octet evidence` subcommand yet: an adapter is per-crate code,
not data, and Octet has no adapter-loading ABI. The imperative shell that owns a
target crate must:

1. load the model bytes, trace set, pins, and adapter;
2. measure cumulative replay time and pass it in `StepObservation`;
3. call `spec_trace_parity::run`; and
4. persist the returned receipt and replay record.

Wiring the rail into `cargo-octet` is deferred until an adapter loading surface
exists. Until then the core is exercised by the committed positive and negative
fixtures in `spec_trace_parity::tests`.

## Fixtures

Positive fixtures cover fresh parity, deterministic replay, projection and loss
bound recording, and seed binding. Negative fixtures cover stale model, stale
checker identity and version, seed drift, tampered traces, unsupported trace
format, divergence, adapter loss, missing adapter projection or loss bound,
step and time bound denials, unsupported verifier, swapped receipt classes,
receipt identity drift, and malformed requests.

## Non-claims

A passing parity rail proves only that the admitted traces matched the
implementation under the pinned model, checker, seed, trace set, and adapter
projection. It does not prove model validity, checker soundness, behavior
outside the traced set, or release eligibility. A divergence is evidence of one
observed mismatch, not a proof about every execution.
