## Context

This change imports a narrow Unison lesson into Slotcar: use content identity and structured tool state where it improves reviewability, but keep Rust, Cargo, Dylint, and Git as the execution substrate.

## Goals / Non-Goals

**Goals:**
- Define the minimum durable contract for `executable-doc-transcripts`.
- Keep outputs deterministic and free of local-only data.
- Preserve current Slotcar command behavior unless the new surface is explicitly requested.

**Non-Goals:**
- No Unison syntax, UCM clone, or distributed runtime.
- No broad semantic-equivalence claims that Slotcar cannot prove.
- No hard CI gate until positive and negative fixtures exist.

## Decisions

### 1. Additive command/output contract

**Choice:** Implement this as additive CLI/output behavior under Slotcar-owned commands and metadata fields.

**Rationale:** Consumers can adopt incrementally without breaking existing check/fingerprint workflows.

**Alternative:** Replace existing output schemas wholesale. Rejected because downstream rollout evidence depends on stable schemas.

### 2. Deterministic receipts and fixtures

**Choice:** Every scenario must be backed by a deterministic fixture or helper transcript saved under the change evidence path during implementation.

**Rationale:** Slotcar proof claims need receipts/gates, not narrative.

**Alternative:** Rely on prose examples only. Rejected because examples drift.

### 3. Explicit unknown/unsupported states

**Choice:** When the tool cannot compute a requested identity/classification/cache key, it records an explicit unsupported status.

**Rationale:** Non-proofs must be named so automation does not overtrust partial data.

**Alternative:** Omit fields silently. Rejected because omission is ambiguous.

## Risks / Trade-offs

**Overclaiming semantic stability** → Mitigate with explicit field names and unsupported statuses.

**Schema churn** → Mitigate through versioned DTOs, tests, and docs updates.

**Consumer confusion** → Mitigate with examples and migration notes before promotion.
