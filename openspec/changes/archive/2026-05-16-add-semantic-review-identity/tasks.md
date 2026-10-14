## Phase 1: Implementation

- [x] I1 Add the core DTOs and parsing/output seams for `semantic-review-identity`
- [x] I2 Add positive and negative fixtures for `semantic-review-identity`
- [x] I3 Update docs and examples for `semantic-review-identity`

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Normalized item identity survives rename" with targeted fixture/helper; save output at `openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-scenario-1.txt`
- [x] V2 Verify scenario "Baseline matching uses semantic identity first" with targeted fixture/helper; save output at `openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-scenario-2.txt`
- [x] V3 Verify scenario "Dependency identity is explicit when unavailable" with targeted fixture/helper; save output at `openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review the implemented semantic-review-identity UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-human-review.txt`

## Change Status
- Status: completed
- Open tasks: none

## Verification Coverage

- `Scenario: Normalized item identity survives rename` -> `V1` -> `openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-scenario-1.txt`
- `Scenario: Baseline matching uses semantic identity first` -> `V2` -> `openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-scenario-2.txt`
- `Scenario: Dependency identity is explicit when unavailable` -> `V3` -> `openspec/changes/add-semantic-review-identity/evidence/add-semantic-review-identity-scenario-3.txt`
