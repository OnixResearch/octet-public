## Phase 1: Spec package

- [x] I1 Create Octet-owned OpenSpec proposal, design, delta spec, and tasks for content-addressed Rust evidence.
- [x] V1 Validate the OpenSpec package and save gate output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/openspec-gates.txt`.

## Phase 2: Function-object v1

- [x] I2 Add function-object v1 data model and canonical JSON/hash helpers for selected functions.
- [x] I3 Add object emission command surface for selected source-scanned functions.
- [x] V2 Verify scenario "Emit source-token function object" with focused cargo-octet tests; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/function-object-v1.txt`.
- [x] V3 Verify scenario "Reject correctness overclaim" with docs/output fixture checks; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/no-correctness-overclaim.txt`.
- [x] V4 Verify scenario "Same logical object hashes the same across roots" with cross-root fixture tests; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/canonical-cross-root.txt`.
- [x] V5 Verify scenario "Entropy-bearing fields are excluded or rejected" with negative canonicalization fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/canonical-entropy.txt`.

## Phase 3: Markers, pointers, and diffs

- [x] I4 Add object marker check/update compatibility while preserving existing fingerprint workflows.
- [x] I5 Add name-pointer manifest support and deterministic pointer move output.
- [x] I6 Add object-aware diff output separating body, signature, caveat, and pointer changes.
- [x] V6 Verify scenario "Object marker check detects drift" with marker drift fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/object-marker-drift.txt`.
- [x] V7 Verify scenario "Existing fingerprint workflows remain usable" with existing fingerprint tests; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/fingerprint-compat.txt`.
- [x] V8 Verify scenario "Pointer move records old and new identity" with pointer manifest fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/name-pointer-move.txt`.
- [x] V9 Verify scenario "Pointer references must resolve" with missing-object negative fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/name-pointer-resolution.txt`.
- [x] V10 Verify scenario "Diff separates body and caveat changes" with old/new object fixture pairs; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/object-diff.txt`.

## Phase 4: Caveats and bundle export

- [x] I7 Add structured dependency, macro/generated-code, and effect caveat vocabulary.
- [x] I8 Add export bundle writer with deterministic manifests and replay metadata.
- [x] V11 Verify scenario "Source-scanned dependencies are not reported as resolved" with function-object metadata fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/dependency-caveats.txt`.
- [x] V12 Verify scenario "Unknown effects block pure-cache eligibility" with unresolved effect fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/effect-caveats.txt`.
- [x] V13 Verify scenario "Bundle membership is deterministic" with repeated export fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/bundle-membership.txt`.
- [x] V14 Verify scenario "Bundle records replay command" with bundle metadata fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/bundle-replay-metadata.txt`.

## Phase 5: Real-code corpus rail

- [x] I10 Add a bounded real-code corpus rail for object emission and bundle export against Octet implementation modules.
- [x] V19 Verify scenario "Real-code corpus emits stable source objects" with cargo-octet/src corpus evidence; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/real-code-corpus.txt`.
- [x] V20 Verify scenario "Real-code corpus bundle is deterministic" with repeated bundle export evidence; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/real-code-bundle.txt`.

## Phase 6: Harness hardening

- [x] I11 Add a reusable real-code corpus harness with named corpus inputs, stable object emission helpers, caveat summary assertions, and deterministic bundle helpers.
- [x] V21 Verify scenario "Repeatable real-code harness emits classified summary" with real-code harness evidence; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/harness-corpus-summary.txt`.
- [x] V22 Verify scenario "Negative corpus cases are classified" with body/comment/macro/pointer negative fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/harness-negative-cases.txt`.
- [x] V23 Verify scenario "Harness bundle export is deterministic" with repeated bundle export and unresolved pointer failure fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/harness-bundle-determinism.txt`.
- [x] V24 Verify scenario "Strong local determinism receipt" with ten-run scrubbed-env emit/bundle checks, fresh target dirs, copied-root equivalence, input-order permutation equivalence, and BLAKE3 receipt hashes; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/strong-local-determinism.txt`.

## Phase 7: IR roadmap and downstream boundaries

- [x] I9 Document HIR/MIR supplemental evidence contract and downstream ownership boundaries.
- [x] V15 Verify scenario "HIR evidence is toolchain scoped" with schema/docs fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/hir-boundary.txt`.
- [x] V16 Verify scenario "MIR evidence does not claim cross-version stability" with schema/docs fixtures; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/mir-boundary.txt`.
- [x] V17 Verify scenario "Cairn consumes Octet receipts without owning extraction" with interop docs/schema checks; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/cairn-boundary.txt`.
- [x] V18 Verify scenario "Crunch and Aspen pilots remain bounded" with interop docs/schema checks; save output at `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/downstream-boundaries.txt`.

## Change Status

- Status: complete pending archive
- Open tasks: none

## Verification Coverage

- `Scenario: Emit source-token function object` -> `V2` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/function-object-v1.txt`
- `Scenario: Reject correctness overclaim` -> `V3` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/no-correctness-overclaim.txt`
- `Scenario: Real-code corpus emits stable source objects` -> `V19` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/real-code-corpus.txt`
- `Scenario: Same logical object hashes the same across roots` -> `V4` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/canonical-cross-root.txt`
- `Scenario: Entropy-bearing fields are excluded or rejected` -> `V5` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/canonical-entropy.txt`
- `Scenario: Object marker check detects drift` -> `V6` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/object-marker-drift.txt`
- `Scenario: Existing fingerprint workflows remain usable` -> `V7` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/fingerprint-compat.txt`
- `Scenario: Pointer move records old and new identity` -> `V8` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/name-pointer-move.txt`
- `Scenario: Pointer references must resolve` -> `V9` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/name-pointer-resolution.txt`
- `Scenario: Source-scanned dependencies are not reported as resolved` -> `V11` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/dependency-caveats.txt`
- `Scenario: Unknown effects block pure-cache eligibility` -> `V12` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/effect-caveats.txt`
- `Scenario: Diff separates body and caveat changes` -> `V10` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/object-diff.txt`
- `Scenario: Bundle membership is deterministic` -> `V13` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/bundle-membership.txt`
- `Scenario: Bundle records replay command` -> `V14` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/bundle-replay-metadata.txt`
- `Scenario: Real-code corpus bundle is deterministic` -> `V20` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/real-code-bundle.txt`
- `Scenario: Repeatable real-code harness emits classified summary` -> `V21` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/harness-corpus-summary.txt`
- `Scenario: Negative corpus cases are classified` -> `V22` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/harness-negative-cases.txt`
- `Scenario: Harness bundle export is deterministic` -> `V23` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/harness-bundle-determinism.txt`
- `Scenario: Strong local determinism receipt` -> `V24` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/strong-local-determinism.txt`
- `Scenario: HIR evidence is toolchain scoped` -> `V15` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/hir-boundary.txt`
- `Scenario: MIR evidence does not claim cross-version stability` -> `V16` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/mir-boundary.txt`
- `Scenario: Cairn consumes Octet receipts without owning extraction` -> `V17` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/cairn-boundary.txt`
- `Scenario: Crunch and Aspen pilots remain bounded` -> `V18` -> `openspec/changes/archive/2026-05-19-content-addressed-rust-evidence/evidence/downstream-boundaries.txt`
