## Phase 1: OpenSpec foundation

- [x] I1 Create the focused child OpenSpec for the public unsafe API lint candidate
- [x] V1 Verify the child OpenSpec validates strictly; evidence=`openspec validate add-public-unsafe-api-lint --strict`
- [x] V2 Verify the new child is the only active OpenSpec change; evidence=`python helper/inventory active changes`

## Phase 2: Implementation

- [x] I2 Implement the `public_unsafe_api` lint in `src/safety/` and register it in the lint inventory
- [x] I3 Add diagnostic metadata, catalog/docs entries, configuration naming, and rollout-profile defaults for the new lint
- [x] I4 Add compliant, noncompliant, and false-positive-boundary UI fixtures for public unsafe free functions, inherent methods, private helpers, FFI shims, and trait-dictated methods

## Phase 3: Verification

- [x] V3 Run focused UI tests for the new fixtures; evidence=`openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/ui-public-unsafe-api.txt`
- [x] V4 Run package-level checks required for lint registration changes; evidence=`openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/package-checks.txt`
- [x] V5 Run policy/export checks proving the lint remains advisory or allow-by-default until fixture and downstream inventory evidence support promotion; evidence=`openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/policy-advisory-gate.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Public unsafe free function warns` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/ui-public-unsafe-api.txt`
- `Scenario: Public unsafe inherent method warns` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/ui-public-unsafe-api.txt`
- `Scenario: Private unsafe helper is accepted` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/ui-public-unsafe-api.txt`
- `Scenario: FFI shim is accepted` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/ui-public-unsafe-api.txt`
- `Scenario: Trait-dictated unsafe method is accepted` -> `I4,V3` -> `openspec/changes/archive/2026-05-04-add-public-unsafe-api-lint/evidence/ui-public-unsafe-api.txt`
