# Concurrency-state checklist template

## Component or protocol boundary

- Artifact under review:
- Owner / reviewer:
- Boundary type (thread, task, lock graph, protocol step, replica path):
- Purpose or contract summary:
- Preconditions or invariants that must fail loudly if broken:

## Shared-state inventory

- Shared mutable state items:
- Single owner or write authority for each item:
- Read-only observers or subscribers:
- Transfer or handoff points:
- State that should stop being shared:

## Lock and peer ordering

- Lock or guard names:
- Required acquisition order:
- Release or downgrade rules:
- Peer / shard / resource ordering rule:
- Path that still lacks an ordering note:

## State-machine transitions and invariants

- Named states:
- Allowed transitions:
- Invariant checked before transition:
- Invariant checked after transition:
- Invalid transition handling:
- Transition or invariant still implicit:

## Delivery, retry, and idempotence assumptions

- Message or event order assumed:
- Delivery model (at-most-once, at-least-once, deduplicated, other):
- Retry trigger or replay source:
- Idempotence key or dedup rule:
- Side effect that must not duplicate:
- Failure mode if ordering assumption breaks:

## Unsafe boundary and escape hatches

- Unsafe block or module:
- Safe wrapper that contains it:
- Invariant defended by wrapper:
- Dangerous escape hatch name:
- Audit note or grep token for the exceptional path:
- Boundary that still needs quarantine:

## Open risks and follow-ups

- Highest-risk ordering or state bug still open:
- Missing test, proof, or review artifact:
- Follow-up owner:
- Suppression, exception, or temporary gap to justify:
- Review verdict / next action:
