# Bounds and pacing review guide

Tiger Style treats bounded allocation and event pacing as design contracts, not cleanup after a production incident. Review who owns memory, queues, and event intake in plain language first so later lints, load tests, and runtime checks defend a named bound instead of wishful thinking.

## Allocation lifetime and ownership

This section matters because "we only allocate a little" turns into leaks, retention, or accidental buffering when no actor owns growth or release. Name which component allocates, who may retain data, when objects are freed or reset, and which path may grow collections after initialization.

- Reviewer prompt: Which actor owns each allocation, buffer, cache, or retained batch on this path?
- Reviewer prompt: What exact event, scope exit, flush, ack, timeout, or restart releases that memory again?
- Checklist: Send work back when memory can grow or remain retained but no owner or release condition is named.

## Startup sizing and post-init bounds

This section matters because systems often need bursts of allocation during startup, replay, or warm-up, but that does not justify open-ended growth forever. Separate what is sized once at boot from what may still grow later, and keep post-init growth bounded and reviewable.

- Reviewer prompt: What state, pools, caches, queues, or tables are intentionally sized during startup, and what number or formula sets that size?
- Reviewer prompt: After initialization completes, which resources may still grow, and what explicit item or byte bound limits them?
- Checklist: Prefer startup sizing for known steady-state structures and require an explicit post-init bound for anything that can still expand.

## Queues, buffers, and backpressure

This section matters because queues hide overload until memory, latency, or downstream work explodes. Every queue, buffer, and channel needs a named bound plus a named response when producers outrun consumers.

- Reviewer prompt: What exact max item count or byte bound exists for each queue, channel, collector, or retry buffer?
- Reviewer prompt: What batch size, flush trigger, or drain policy amortizes work without letting latency or residency drift upward?
- Reviewer prompt: When producers outrun consumers, what backpressure, shedding, blocking, or failure path keeps backlog bounded?
- Checklist: Every queue gets a bound, every drain path gets a batching rule, and every overload path says what happens next.

## External-event pacing

This section matters because systems fail when they react directly to every external interrupt, packet, timer, or callback at arrival speed. Buffer external events first, then let the system run at its own pace so one hot source cannot dictate memory growth or scheduler behavior.

- Reviewer prompt: Which component owns intake pace for each external event source, and where does it buffer before processing?
- Reviewer prompt: What polling, dequeue, credit, or token rule makes the system run at its own pace instead of reacting directly to every event?
- Reviewer prompt: If an external source spikes or stalls, how does the system preserve ordering, fairness, or bounded lag without turning the source into the scheduler?
- Checklist: Prefer buffer-then-process boundaries with one named owner for intake pace, batching, and drain rules.

## Batch and work budgeting

This section matters because even bounded queues can still starve the rest of the system if one tick, batch, or loop tries to do unbounded work. Name how much work one pass may do before yielding, flushing, or rescheduling.

- Reviewer prompt: What is the explicit per-batch, per-tick, or per-loop work budget in items, bytes, time, or CPU steps?
- Reviewer prompt: What condition ends the current batch and yields to timers, I/O, heartbeats, or other tenants?
- Reviewer prompt: Which hot path should coalesce work into larger chunks, and which path must stay latency-sensitive even if throughput drops?
- Checklist: Keep one obvious budget per work loop and reject designs that say "drain until empty" without a stronger bound.

## Overload and failure behavior

This section matters because bounds only help when overload behavior is deliberate. When inputs exceed budget, the system needs a named posture: block, shed, retry later, degrade service, fail fast, or preserve only a smaller safety-critical core.

- Reviewer prompt: When a queue fills, a batch times out, or memory hits its cap, does the system block, shed, retry later, degrade service, or fail fast?
- Reviewer prompt: Which invariants must still hold under overload, and what data loss, latency increase, or feature disablement is acceptable?
- Reviewer prompt: Does the chosen failure path make the problem visible quickly, or does it hide trouble behind silent buffering and retries?
- Checklist: Overload plans must name trigger, behavior, operator signal, and recovery path.

## Reviewer prompts and escape hatches

This checklist matters because boundedness rules keep incidents legible before they become paging noise. The goal is not to ban every exception; it is to make each exception narrow, audited, and time-bounded so reviewers can tell whether an unbounded path is a real design choice or an accidental omission.

- Reviewer prompt: If a path is intentionally unbounded or directly reactive, where is the audited exception, who approved it, and what makes the blast radius acceptable?
- Reviewer prompt: What exact review cadence revisits this exception or bound as workload, hardware, or deployment shape changes?
- Reviewer prompt: If a reviewer cannot explain ownership, bounds, batching, backpressure, and pacing in one paragraph, what design note or code change is missing?
- Checklist: Any escape hatch names owner, scope, rationale, monitoring signal, and next review date.
- Checklist: Send work back when bounds exist only by convention, when external events still drive processing directly, or when exceptions have no audit trail or re-review date.
