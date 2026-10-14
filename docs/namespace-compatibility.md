# Namespace compatibility

Octet treats current product names as canonical and legacy names as bounded compatibility rows. A legacy alias row records the canonical identity, legacy spelling, allowed surface, owner, status, retirement condition, and the non-claim that alias approval preserves compatibility only; it does not make the legacy identity canonical.

## Alias statuses

- `approved`: a compatibility surface may keep the alias while the migration remains active.
- `deprecated`: the alias is still recognized for a documented transition path, but new active docs should prefer the canonical identity.
- `archive-only`: the alias may appear in archived lifecycle evidence or historical replay paths only.
- `blocked`: the alias must not appear in active or compatibility surfaces.

## Retirement expectations

Every alias row needs a retirement condition. Missing retirement rationale is a validation failure because reviewers cannot distinguish intentional compatibility from identity drift. Passing alias validation does not prove consumer compatibility, release eligibility, semantic equivalence, or project ownership transfer.
