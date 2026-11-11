## Context

`cargo octet object corpus receipt` already records aggregate caveat counts and pure-cache blockers. The expanded internal corpus taxonomy fixture records selected source paths, but there is no operator-facing report that connects caveat distribution back to each selected source bucket.

## Goals

- Add a deterministic report derived from existing Function Object v1 records.
- Group by normalized source path so the report remains stable and reviewable.
- Preserve no-correctness/no-semantic-equivalence/no-pure-cache-promotion boundaries.

## Non-Goals

- Do not add HIR/MIR extraction.
- Do not classify semantic effects beyond existing source-token caveats.
- Do not alter object hashes or Function Object v1 identity.

## Decisions

### 1. Add a corpus subcommand

**Choice:** Add `cargo octet object corpus caveats --output-format human|json [PATHS...]`.

**Rationale:** It reuses the existing corpus input model and avoids changing receipt schema v1.

### 2. Group by source path

**Choice:** Emit `source_buckets` sorted by normalized source path, with per-bucket object count, caveat counts, pure-cache blocked count, and blocker names.

**Rationale:** Source path is already the selected-corpus bucket in existing receipts and taxonomy fixtures.

### 3. Keep boundaries explicit

**Choice:** Reuse corpus boundary wording and add report detail that caveat reports explain blockers only.

**Rationale:** This prevents accidental promotion from review evidence to correctness or pure-cache proof.

## Risks / Mitigations

- Fixture churn when `object.rs` changes: refresh internal and expanded corpus receipts in the same review.
- Overclaim risk: tests assert boundary booleans remain false and blocker report is descriptive only.
