## Phase 1: Implementation

- [x] [serial] I1 Implement review identity registry DTOs and target hash normalization
- [x] [depends:I1] I2 Extend fingerprint review manifests with label and immutable target fields
- [x] [depends:I1] I3 Implement pointer history inspection and rollback behavior
- [x] [depends:I2] I4 Implement stale, missing, and mismatched identity diagnostics
- [x] [depends:I3,I4] I5 Document review identity operator workflows and no-correctness-claim language

## Phase 2: Deterministic verification

- [x] [depends:I2] V1 Verify scenario "Label resolves to immutable review target" with review manifest fixture; save output at `openspec/changes/add-slotcar-review-identity-registry/evidence/label-resolves-target.txt`
- [x] [depends:I3] V2 Verify scenario "Pointer history supports rollback" with pointer history fixture; save output at `openspec/changes/add-slotcar-review-identity-registry/evidence/pointer-rollback.txt`
- [x] [depends:I4] V3 Verify scenario "Missing identity is diagnosed" with missing-label fixture; save output at `openspec/changes/add-slotcar-review-identity-registry/evidence/missing-identity.txt`
- [x] [depends:I4] V4 Verify scenario "Identity match avoids correctness claim" with output wording fixture; save output at `openspec/changes/add-slotcar-review-identity-registry/evidence/no-correctness-claim.txt`
- [x] [depends:I5] V5 Verify scenario "Guidance explains name and hash roles" with docs scan; save output at `openspec/changes/add-slotcar-review-identity-registry/evidence/docs-name-hash.txt`
- [x] [depends:I5] V6 Verify scenario "Guidance documents rollback" with docs-example check; save output at `openspec/changes/add-slotcar-review-identity-registry/evidence/docs-rollback.txt`

## Phase 3: Human/oracle checkpoints

- [x] [depends:V1,V2,V3,V4,V5,V6] H1 Human/oracle checkpoint: command=`review add-slotcar-review-identity-registry against Slotcar fingerprint policy and Unison name/hash lessons`; input=`proposal.md, design.md, specs/standards-codification/spec.md, specs/documentation/spec.md`; asserts=`registry separates mutable labels from immutable evidence identity without requiring blanket fingerprints or overclaiming correctness`; evidence=`openspec/changes/add-slotcar-review-identity-registry/evidence/scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Label resolves to immutable review target` -> `V1` -> `openspec/changes/add-slotcar-review-identity-registry/evidence/label-resolves-target.txt`
- `Scenario: Pointer history supports rollback` -> `V2` -> `openspec/changes/add-slotcar-review-identity-registry/evidence/pointer-rollback.txt`
- `Scenario: Missing identity is diagnosed` -> `V3` -> `openspec/changes/add-slotcar-review-identity-registry/evidence/missing-identity.txt`
- `Scenario: Identity match avoids correctness claim` -> `V4` -> `openspec/changes/add-slotcar-review-identity-registry/evidence/no-correctness-claim.txt`
- `Scenario: Guidance explains name and hash roles` -> `V5` -> `openspec/changes/add-slotcar-review-identity-registry/evidence/docs-name-hash.txt`
- `Scenario: Guidance documents rollback` -> `V6` -> `openspec/changes/add-slotcar-review-identity-registry/evidence/docs-rollback.txt`
