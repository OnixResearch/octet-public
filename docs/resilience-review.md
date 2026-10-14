# Resilience review guide

Tiger Style treats resilience as a design contract, not cleanup after an outage or an excuse to add more retries. Review symptoms, blast radius, retry assumptions, and failure posture in plain language first so later tests, runbooks, and recovery code defend a named design choice instead of cargo-cult incident handling.

## Incident, symptom, and immediate workaround

This section matters because resilience review starts from a real or plausible failure, not from abstract optimism. Name the observed symptom, who noticed it, what users or operators saw, and which short-term workaround restored service without pretending the workaround explains the cause.

- Reviewer prompt: What exact symptom, alert, user-visible failure, or operational pain triggered this review?
- Reviewer prompt: What immediate workaround, restart, rollback, retry, or manual intervention restored service, and what did that workaround leave unexplained?
- Checklist: Record the symptom and workaround in concrete terms before debating architecture so the review stays anchored to the actual failure.

## Five whys to the design decision

This section matters because repeated incidents usually come from a design choice, not from the final visible symptom. Climb from the symptom to the architectural decision, boundary, or assumption underneath it, and do not stop at "we retried" or "we restarted it" as if that were root cause.

- Reviewer prompt: Starting from the symptom, what is the five-whys chain that reaches a design decision, default, coupling, or ownership gap?
- Reviewer prompt: Which why in the chain first shifts from operational noise to an architectural cause that maintainers can change?
- Checklist: Send work back when the chain stops at restart, retry, or operator action instead of naming the design decision underneath the incident.

## Boundary, blast radius, and affected invariants

This section matters because resilience work goes sideways when the failure boundary is vague. Name which component, dependency, queue, shard, tenant set, or API boundary failed, how far the damage spread, and which invariants or guarantees were threatened.

- Reviewer prompt: What exact boundary contains this failure, and what data, tenants, peers, or downstream systems sit inside its blast radius?
- Reviewer prompt: Which invariants, ordering guarantees, durability promises, or safety checks were violated or came close to violation?
- Checklist: Keep boundary and blast-radius notes explicit enough that reviewers can tell what stayed healthy and what could have failed next.

## Retry, replay, idempotence, and degradation

This section matters because retries can heal one class of transient failure while quietly amplifying duplication, overload, or stale-state bugs. Review what repeats, what replays, what must be idempotent, and what degraded mode is acceptable before adding another recovery loop.

- Reviewer prompt: What request, job, message, timer, or operator action may retry or replay on this path?
- Reviewer prompt: What makes repeated delivery safe here: idempotence key, dedup rule, sequence boundary, compensating action, or explicit non-repeatable contract?
- Reviewer prompt: If the system cannot deliver full service, what degraded posture is acceptable and which guarantees must still hold?
- Checklist: Every retry or replay path names its safety condition, and every degraded mode says what the system still promises.

## Fail-open vs fail-closed

This section matters because resilience choices are not neutral. When a dependency misbehaves or a safety check cannot complete, the system still chooses a posture: continue with reduced assurance, stop the operation, or narrow service to a safer subset.

- Reviewer prompt: For this failure, should the system fail open, fail closed, or degrade into a smaller explicitly named mode, and why?
- Reviewer prompt: What user harm, data risk, security risk, or availability loss changes if the posture flips the other way?
- Checklist: Name the chosen failure posture explicitly and reject designs that drift into fail-open or fail-closed behavior by accident.

## Reviewer prompts and follow-ups

This section matters because resilience review needs a disposition, not a comforting story. The goal is to leave with an owner, a follow-up, and a clearer design rule, while keeping the standard explicit that restart or retry alone is not root cause.

- Reviewer prompt: What design change, ownership fix, observability addition, or test should happen before this incident pattern is considered addressed?
- Reviewer prompt: Who owns the follow-up, and what evidence will show the design decision changed instead of the workaround merely getting faster?
- Reviewer prompt: If a reviewer cannot explain the symptom, boundary, retry assumptions, and failure posture in one paragraph, what note or code change is still missing?
- Checklist: Restart or retry alone is not root cause; the review must name the design decision that made the symptom repeatable.
- Checklist: Send work back when follow-up ownership is vague, degraded mode is implicit, or the guide leaves the incident as oral tradition instead of a reviewable contract.
