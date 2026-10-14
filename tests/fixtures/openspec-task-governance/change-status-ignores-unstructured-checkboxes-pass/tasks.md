## Phase 1: Implementation

- [x] I1 add governed-task example
- [ ] I2 leave one governed implementation task open
- [ ] note follow-up prose-only checkbox that should stay ungoverned

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Unstructured checkboxes do not change status" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh change-status-ignores-unstructured-checkboxes-pass`; save output at `openspec/changes/change-status-ignores-unstructured-checkboxes-pass/evidence/change-status-unstructured-pass.txt`
- [x] note checked prose-only checkbox that should stay ungoverned

## Change Status
- Status: in-progress
- Open tasks: I2, V1
## Verification Coverage

- `Scenario: Unstructured checkboxes do not change status` -> `V1` -> `openspec/changes/change-status-ignores-unstructured-checkboxes-pass/evidence/change-status-unstructured-pass.txt`
