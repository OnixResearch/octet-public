## Phase 1: Implementation

- [x] [serial] I1 Add typed capability vocabulary data and validation rules
- [x] [depends:I1] I2 Map existing effect-like lint and policy entries to capability families
- [x] [depends:I2] I3 Expose capability families in JSON, SARIF, and human summaries
- [x] [depends:I1] I4 Document FCIS authority classification and advisory/hard promotion rules

## Phase 2: Deterministic verification

- [x] [depends:I1,I2] V1 Verify scenario "Policy registry names capability family" with registry export fixture; save output at `openspec/changes/add-slotcar-effect-capability-policy/evidence/registry-family.txt`
- [x] [depends:I4] V2 Verify scenario "FCIS guidance classifies core and shell authority" with docs scan; save output at `openspec/changes/add-slotcar-effect-capability-policy/evidence/fcis-guidance.txt`
- [x] [depends:I3] V3 Verify scenario "Known capability appears in output" with mapped finding fixture; save output at `openspec/changes/add-slotcar-effect-capability-policy/evidence/known-capability-output.txt`
- [x] [depends:I1] V4 Verify scenario "Unknown capability is visible" with unmapped-lint fixture; save output at `openspec/changes/add-slotcar-effect-capability-policy/evidence/unknown-capability.txt`
- [x] [depends:I3] V5 Verify scenario "Consumer output groups findings by capability" with downstream JSON/human fixture; save output at `openspec/changes/add-slotcar-effect-capability-policy/evidence/consumer-groups.txt`
- [x] [depends:I3] V6 Verify scenario "Consumer output preserves advisory status" with advisory capability fixture; save output at `openspec/changes/add-slotcar-effect-capability-policy/evidence/advisory-status.txt`

## Phase 3: Human/oracle checkpoints

- [x] [depends:V1,V2,V3,V4,V5,V6] H1 Human/oracle checkpoint: command=`review add-slotcar-effect-capability-policy for Unison-ability inspiration without Rust language-effect overreach`; input=`proposal.md, design.md, specs/standards-codification/spec.md, specs/consumer-integration/spec.md`; asserts=`capability policy makes effects reviewable while keeping enforcement fixture-backed and non-magical`; evidence=`openspec/changes/add-slotcar-effect-capability-policy/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Policy registry names capability family` -> `V1` -> `openspec/changes/add-slotcar-effect-capability-policy/evidence/registry-family.txt`
- `Scenario: FCIS guidance classifies core and shell authority` -> `V2` -> `openspec/changes/add-slotcar-effect-capability-policy/evidence/fcis-guidance.txt`
- `Scenario: Known capability appears in output` -> `V3` -> `openspec/changes/add-slotcar-effect-capability-policy/evidence/known-capability-output.txt`
- `Scenario: Unknown capability is visible` -> `V4` -> `openspec/changes/add-slotcar-effect-capability-policy/evidence/unknown-capability.txt`
- `Scenario: Consumer output groups findings by capability` -> `V5` -> `openspec/changes/add-slotcar-effect-capability-policy/evidence/consumer-groups.txt`
- `Scenario: Consumer output preserves advisory status` -> `V6` -> `openspec/changes/add-slotcar-effect-capability-policy/evidence/advisory-status.txt`
