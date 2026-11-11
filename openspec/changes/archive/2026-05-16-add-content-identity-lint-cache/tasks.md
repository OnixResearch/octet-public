## Phase 1: Implementation

- [x] I1 Add the core DTOs and parsing/output seams for `content-identity-lint-cache`
- [x] I2 Add positive and negative fixtures for `content-identity-lint-cache`
- [x] I3 Update docs and examples for `content-identity-lint-cache`

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Unchanged item uses cached finding" with targeted fixture/helper; save output at `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-1.txt`
- [x] V2 Verify scenario "Config change invalidates cache" with targeted fixture/helper; save output at `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-2.txt`
- [x] V3 Verify scenario "Non-deterministic input bypasses cache" with targeted fixture/helper; save output at `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review the implemented content-identity-lint-cache UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-human-review.txt`

## Change Status
- Status: completed
- Open tasks: none

## Verification Coverage

- `Scenario: Unchanged item uses cached finding` -> `V1` -> `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-1.txt`
- `Scenario: Config change invalidates cache` -> `V2` -> `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-2.txt`
- `Scenario: Non-deterministic input bypasses cache` -> `V3` -> `openspec/changes/add-content-identity-lint-cache/evidence/add-content-identity-lint-cache-scenario-3.txt`
