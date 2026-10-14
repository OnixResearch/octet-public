# Five-whys architecture template

## Incident or scenario under review

- Artifact under review:
- Owner / reviewer:
- Incident date or scenario trigger:
- Component, service, or boundary under review:
- Short summary of the resilience concern:

## Symptom and immediate workaround

- User-visible symptom or operator signal:
- First alert, report, or observed failure:
- Immediate workaround, restart, retry, rollback, or manual step:
- What the workaround restored:
- What the workaround did not explain or fix:

## Five whys chain

- Why 1 (symptom to immediate cause):
- Why 2:
- Why 3:
- Why 4:
- Why 5:
- Design decision, assumption, or ownership gap reached by the chain:
- Point where the chain would wrongly stop at retry or restart if left unchallenged:

## Boundary, blast radius, and affected invariants

- Boundary that contains this failure:
- Blast radius (users, tenants, shards, peers, downstream systems):
- Invariants or guarantees threatened:
- What stayed healthy outside the boundary:
- Next boundary that would fail if the issue spread:

## Retry, replay, idempotence, and degradation assumptions

- Operation, message, or job that may retry or replay:
- Retry trigger or replay source:
- Idempotence key, dedup rule, or reason repeats are safe:
- Side effect that must not duplicate:
- Acceptable degraded mode if full behavior cannot continue:
- Guarantee that must still hold during degradation:
- Assumption here that still lacks proof or instrumentation:

## Fail-open vs fail-closed decision

- Chosen posture (fail-open, fail-closed, degraded mode):
- Why this posture is safer than the alternative here:
- Availability risk if the system fails closed:
- Correctness, safety, or security risk if the system fails open:
- Trigger that moves the system into this posture:
- Condition that allows recovery to normal behavior:

## Follow-ups, owners, and non-goals

- Design change or guardrail to add:
- Test, alert, runbook, or observability follow-up:
- Owner for each follow-up:
- Non-goals for this review:
- Residual risk accepted for now:
- Review verdict / next action:
