## Context

The object corpus checker already distinguishes shape errors, source-selection errors, parse errors, and field mismatches. Compatibility fixtures currently preserve only the positive v1 shape, a missing required field, and an unsupported future schema version.

## Goals / Non-Goals

**Goals:**
- Preserve fail-closed behavior for malformed receipts, duplicate source paths, boundary tamper, empty selections, and unsupported non-Rust selections.
- Keep examples deterministic and local to the existing compatibility fixture directory.
- Keep existing v1 schema and check-report contracts unchanged.

**Non-Goals:**
- Add migration support or accept any new receipt schema.
- Treat object hashes as correctness or semantic-equivalence proofs.
- Expand to downstream corpora.

## Decisions

### 1. Static negative receipt fixtures plus pinned source-selection cases

**Choice:** Store malformed, duplicate-source-path, and boundary-tamper receipts as checked-in negative fixtures; store `notes.txt` as the non-Rust source-selection fixture; use the existing positive receipt for the empty-selection replay case.

**Rationale:** Receipt-shape regressions should be inspectable as files, while empty and non-Rust source-selection failures are properties of command inputs rather than receipt JSON alone.

**Alternative:** Generate all negative cases dynamically in tests. Rejected because prior compatibility work intentionally made schema examples reviewable as fixtures.

### 2. Keep boundary tamper as mismatch, not invalid schema

**Choice:** Boundary tampering is expected to produce `status=mismatch` with `field=boundary`, while malformed/duplicate/selection failures produce `status=invalid`.

**Rationale:** A tampered boundary remains valid JSON/schema shape but fails replay comparison. This preserves the distinction between invalid inputs and drift/tamper mismatch reports.

## Risks / Trade-offs

**Fixture drift** -> The tests replay the fixtures directly and assert exact failure classes.

**Overclaiming** -> No new semantic claims are introduced; fixtures only document accepted/rejected v1 receipt shapes and selected fail-closed inputs.
