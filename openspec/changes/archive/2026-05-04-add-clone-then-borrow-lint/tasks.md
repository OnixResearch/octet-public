## Phase 1: OpenSpec foundation

- [x] I1 Create the focused child OpenSpec for the clone-then-borrow lint candidate
- [x] V1 Verify the child OpenSpec validates strictly; evidence=`openspec validate add-clone-then-borrow-lint --strict`
- [x] V2 Verify the new child is the only active OpenSpec change; evidence=`python helper/inventory active changes`

## Phase 2: Implementation

- [x] I2 Implement the `clone_then_borrow` lint in `src/performance/` and register it in the lint inventory
- [x] I3 Add diagnostic metadata, catalog/docs entries, configuration naming, and rollout-profile defaults for the new lint
- [x] I4 Add compliant, noncompliant, and false-positive-boundary UI fixtures for immediate borrowed clone expressions, owned/moved clones, stored clones, mutation/lifetime-detach cases, and macro/expansion boundaries

## Phase 3: Verification

- [x] V3 Run focused UI tests for the new fixtures; evidence=`openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/ui-clone-then-borrow.txt`
- [x] V4 Run package-level checks required for lint registration changes; evidence=`openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/package-checks.txt`
- [x] V5 Run policy/export checks proving the lint remains advisory or allow-by-default until fixture and downstream inventory evidence support promotion; evidence=`openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/policy-advisory-gate.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Immediate String clone then borrow` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/ui-clone-then-borrow.txt`
- `Scenario: Immediate Vec clone then borrow` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/ui-clone-then-borrow.txt`
- `Scenario: Owned clone transfer is accepted` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/ui-clone-then-borrow.txt`
- `Scenario: Stored or lifetime-detaching clone is accepted` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/ui-clone-then-borrow.txt`
- `Scenario: Expansion boundary is skipped` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-clone-then-borrow-lint/evidence/ui-clone-then-borrow.txt`
