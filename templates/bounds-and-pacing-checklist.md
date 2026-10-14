# Bounds and pacing checklist template

## Component or boundary

- Artifact under review:
- Owner / reviewer:
- Boundary type (service, queue, buffer, batch loop, ingress path, other):
- Purpose or contract summary:
- External event sources entering this boundary:

## Allocation plan

- Long-lived allocations or retained state:
- Allocation owner for each item:
- Release, reset, or eviction trigger for each item:
- Path allowed to grow memory after initialization:
- Allocation that still lacks a clear owner or release rule:

## Startup sizing and post-init limits

- Startup-sized structures and chosen size:
- Formula, workload sketch, or replay assumption behind that size:
- Resources still allowed to grow after initialization:
- Explicit post-init item or byte limits:
- Limit that still depends on guesswork:

## Queue and buffer bounds

- Queue, buffer, or channel names:
- Max items or bytes for each:
- Backpressure choice (block, shed, retry later, fail, other):
- Batch size, flush trigger, or drain rule:
- Queue that still lacks a visible bound:

## External-event intake and pacing

- External event source under review:
- Buffer-before-process boundary:
- Component that owns intake pace:
- Rule that lets the system run at its own pace instead of reacting directly to every event:
- Ordering, fairness, or lag constraint to preserve:
- Source that still pushes work directly into processing:

## Batch and work budget

- Per-batch, per-tick, or per-loop work budget:
- Budget unit (items, bytes, time, CPU steps, other):
- Yield, reschedule, or flush trigger:
- Hot path that should coalesce work into a larger chunk:
- Path that still says `drain until empty` or equivalent:

## Overload and failure behavior

- Overload trigger or saturation signal:
- Behavior at the limit (block, shed, degrade, fail fast, other):
- Invariants that must still hold under overload:
- Operator signal, metric, or alert:
- Recovery path after overload:
- Failure mode still undefined:

## Open risks and follow-ups

- Highest-risk bound or pacing gap still open:
- Temporary exception or audited escape hatch:
- Owner for follow-up:
- Re-review date or cadence:
- Review verdict / next action:
