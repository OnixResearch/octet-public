## Phase 1: Implementation

- [x] I1 Add the core DTOs and parsing/output seams for `effect-surface-reporting`
- [x] I2 Add positive and negative fixtures for `effect-surface-reporting`
- [x] I3 Update docs and examples for `effect-surface-reporting`

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "JSON findings include effect family" with targeted fixture/helper; save output at `openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-scenario-1.txt`
- [x] V2 Verify scenario "Human summary aggregates effects" with targeted fixture/helper; save output at `openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-scenario-2.txt`
- [x] V3 Verify scenario "Unknown lints stay explicit" with targeted fixture/helper; save output at `openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review the implemented effect-surface-reporting UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-human-review.txt`

## Change Status
- Status: completed
- Open tasks: none

## Verification Coverage

- `Scenario: JSON findings include effect family` -> `V1` -> `openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-scenario-1.txt`
- `Scenario: Human summary aggregates effects` -> `V2` -> `openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-scenario-2.txt`
- `Scenario: Unknown lints stay explicit` -> `V3` -> `openspec/changes/add-effect-surface-reporting/evidence/add-effect-surface-reporting-scenario-3.txt`
