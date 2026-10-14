## Phase 1: Implementation

- [x] I1 Add the core DTOs and parsing/output seams for `executable-doc-transcripts`
- [x] I2 Add positive and negative fixtures for `executable-doc-transcripts`
- [x] I3 Update docs and examples for `executable-doc-transcripts`

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Transcript executes command blocks" with targeted fixture/helper; save output at `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-1.txt`
- [x] V2 Verify scenario "Expected output assertion fails closed" with targeted fixture/helper; save output at `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-2.txt`
- [x] V3 Verify scenario "Transcript artifact is content addressed" with targeted fixture/helper; save output at `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-3.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review the implemented executable-doc-transcripts UX and schema`; input=`docs, fixtures, and JSON/SARIF/human output artifacts`; asserts=`the feature is understandable without Unison-specific vocabulary`; evidence=`openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-human-review.txt`

## Change Status
- Status: completed
- Open tasks: none

## Verification Coverage

- `Scenario: Transcript executes command blocks` -> `V1` -> `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-1.txt`
- `Scenario: Expected output assertion fails closed` -> `V2` -> `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-2.txt`
- `Scenario: Transcript artifact is content addressed` -> `V3` -> `openspec/changes/add-executable-doc-transcripts/evidence/add-executable-doc-transcripts-scenario-3.txt`
