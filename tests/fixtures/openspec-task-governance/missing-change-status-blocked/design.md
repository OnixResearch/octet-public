## Context

This fixture should fail only because active `tasks.md` omits `## Change Status`.

## Goals / Non-Goals

**Goals:**
- keep one governed verification task and one matching coverage row
- omit the status section entirely

**Non-Goals:**
- malformed replay fields
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Omit the whole status section

**Choice:** Leave `## Verification Coverage` in place but omit `## Change Status`.

**Rationale:** The gate should emit the exact missing-status finding.

**Implementation:** The fixture keeps one verification row so the missing
status section is the only intended failure.
