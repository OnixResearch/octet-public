## Phase 1: Implementation

- [x] [serial] I1 Implement the replay-capsule DTO, hash normalization, and redaction allowlist
- [x] [depends:I1] I2 Implement capture for `cargo slotcar check` runs
- [x] [depends:I1] I3 Implement capture for `cargo slotcar fingerprint` runs
- [x] [depends:I2,I3] I4 Implement capsule inspect and replay status reporting
- [x] [depends:I4] I5 Document operator capture, inspect, replay, and limit semantics

## Phase 2: Deterministic verification

- [x] [depends:I2] V1 Verify scenario "Capture pins check run identity" with a check-run capsule fixture; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/check-capsule-capture.txt`
- [x] [depends:I3] V2 Verify scenario "Capture pins fingerprint run identity" with a fingerprint capsule fixture; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/fingerprint-capsule-capture.txt`
- [x] [depends:I4] V3 Verify scenario "Replay accepts exact match" with a same-input replay fixture; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/replay-exact-match.txt`
- [x] [depends:I4] V4 Verify scenario "Replay reports drift" with a mutated-input replay fixture; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/replay-drift.txt`
- [x] [depends:I4] V5 Verify scenario "Replay accepts expected failure" with an expected-failure fixture; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/replay-expected-failure.txt`
- [x] [depends:I5] V6 Verify scenario "Operator guide names capture and replay commands" with docs-example check; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/docs-commands.txt`
- [x] [depends:I5] V7 Verify scenario "Operator guide states replay limits" with docs wording check; save output at `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/docs-limits.txt`

## Phase 3: Human/oracle checkpoints

- [x] [depends:V1,V2,V3,V4,V5,V6,V7] H1 Human/oracle checkpoint: command=`review add-slotcar-replay-capsules proposal/design/specs against Slotcar evidence and Unison transcript constraints`; input=`proposal.md, design.md, specs/consumer-integration/spec.md, specs/documentation/spec.md`; asserts=`capsules prove reproducible Slotcar evidence without replacing JSON/SARIF/fingerprint outputs or overclaiming correctness`; evidence=`openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Capture pins check run identity` -> `V1` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/check-capsule-capture.txt`
- `Scenario: Capture pins fingerprint run identity` -> `V2` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/fingerprint-capsule-capture.txt`
- `Scenario: Replay accepts exact match` -> `V3` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/replay-exact-match.txt`
- `Scenario: Replay reports drift` -> `V4` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/replay-drift.txt`
- `Scenario: Replay accepts expected failure` -> `V5` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/replay-expected-failure.txt`
- `Scenario: Operator guide names capture and replay commands` -> `V6` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/docs-commands.txt`
- `Scenario: Operator guide states replay limits` -> `V7` -> `openspec/changes/archive/2026-05-17-add-slotcar-replay-capsules/evidence/docs-limits.txt`
