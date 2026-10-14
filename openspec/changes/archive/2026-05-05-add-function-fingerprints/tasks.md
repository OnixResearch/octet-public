## Phase 1: Implementation

- [x] I1 Add function fingerprint standards documentation and policy registry entries
- [x] I2 Add normalized Rust function token hashing and marker parsing core
- [x] I3 Add `cargo tigerstyle fingerprint --check|--write|--update|--diff` command surface
- [x] I4 Add deterministic machine output for fingerprint validation and inventory records
- [x] I5 Add attribute/comment marker examples and consumer documentation

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Critical function categories are documented" with documentation/policy coverage checks; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/critical-categories.txt`
- [x] V2 Verify scenario "Fingerprints do not claim correctness" with documentation/diagnostic wording checks; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/identity-not-correctness.txt`
- [x] V3 Verify scenario "Missing review class is rejected" with a negative metadata fixture; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/missing-review-class.txt`
- [x] V4 Verify scenario "Review metadata links to evidence" with inventory-output fixture checks; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/review-metadata-evidence.txt`
- [x] V5 Verify scenario "Formatting and comments do not change hash" with paired stable-hash fixtures; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/formatting-stable.txt`
- [x] V6 Verify scenario "Signature or body token changes update hash" with mismatch fixtures; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/token-change-detected.txt`
- [x] V7 Verify scenario "Macro call warns by default" with macro-call fixture diagnostics; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/macro-warns-by-default.txt`
- [x] V8 Verify scenario "Explicit macro allowance suppresses caveat" with macro-allowance fixture diagnostics; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/macro-explicit-allowance.txt`
- [x] V9 Verify scenario "Check reports matching fingerprints" with CLI success fixture; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/check-matching.txt`
- [x] V10 Verify scenario "Check fails on mismatch" with CLI failure fixture; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/check-mismatch.txt`
- [x] V11 Verify scenario "Write mode updates markers only" with before/after token-preservation checks; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/write-markers-only.txt`
- [x] V12 Verify scenario "Diff mode lists reviewed changes" with CLI diff fixture; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/diff-reviewed-functions.txt`
- [x] V13 Verify scenario "Attribute marker validates" with attribute marker fixture; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/attribute-validates.txt`
- [x] V14 Verify scenario "Comment marker validates without proc macro" with stable comment-marker fixture; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/comment-validates.txt`
- [x] V15 Verify scenario "Machine output captures mismatch identity" with JSON-output fixture checks; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/machine-mismatch-identity.txt`
- [x] V16 Verify scenario "Inventory preserves evidence references" with deterministic inventory-output fixture checks; save output at `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/inventory-evidence.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review function fingerprint docs/design against content-addressed-code goal and Rust stable-attribute constraints`; input=`proposal.md, design.md, specs/standards-codification/spec.md, specs/consumer-integration/spec.md`; asserts=`fingerprints are scoped as review identity gates rather than correctness proofs or blanket workspace requirements`; evidence=`openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/fingerprint-scope-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Critical function categories are documented` -> `V1` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/critical-categories.txt`
- `Scenario: Fingerprints do not claim correctness` -> `V2` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/identity-not-correctness.txt`
- `Scenario: Missing review class is rejected` -> `V3` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/missing-review-class.txt`
- `Scenario: Review metadata links to evidence` -> `V4` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/review-metadata-evidence.txt`
- `Scenario: Formatting and comments do not change hash` -> `V5` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/formatting-stable.txt`
- `Scenario: Signature or body token changes update hash` -> `V6` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/token-change-detected.txt`
- `Scenario: Macro call warns by default` -> `V7` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/macro-warns-by-default.txt`
- `Scenario: Explicit macro allowance suppresses caveat` -> `V8` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/macro-explicit-allowance.txt`
- `Scenario: Check reports matching fingerprints` -> `V9` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/check-matching.txt`
- `Scenario: Check fails on mismatch` -> `V10` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/check-mismatch.txt`
- `Scenario: Write mode updates markers only` -> `V11` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/write-markers-only.txt`
- `Scenario: Diff mode lists reviewed changes` -> `V12` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/diff-reviewed-functions.txt`
- `Scenario: Attribute marker validates` -> `V13` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/attribute-validates.txt`
- `Scenario: Comment marker validates without proc macro` -> `V14` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/comment-validates.txt`
- `Scenario: Machine output captures mismatch identity` -> `V15` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/machine-mismatch-identity.txt`
- `Scenario: Inventory preserves evidence references` -> `V16` -> `openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/inventory-evidence.txt`
