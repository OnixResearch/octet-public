## Phase 1: Implementation

- [x] I1 Add external source incorporation classification entries to the standards policy/docs surface
- [x] I2 Add Valgrind-style dynamic evidence profile documentation and review template coverage
- [x] I3 Add Rust idiom guidance for selected Rust Design Patterns concepts and anti-patterns
- [x] I4 Add policy roadmap rows for future lint candidates without promoting hard gates prematurely
- [x] I5 Add functional API boundary guidance for generics/type-state, optics/visitor adapter seams, and fold-heavy style limits

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "External concepts are classified before incorporation" with policy/docs inventory checks; save output at `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/external-source-classification.txt`
- [x] V2 Verify scenario "Dynamic profile guidance names run timing and acceptance signal" with documentation grep/check helper; save output at `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/dynamic-profile-guidance.txt`
- [x] V3 Verify scenario "Heavyweight tools remain optional until a profile is selected" with Nix/check dependency inspection; save output at `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/dynamic-profile-optional-tools.txt`
- [x] V4 Verify scenario "Rust idiom guidance maps each imported pattern to Tiger Style outcomes" with checklist coverage helper; save output at `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/rust-idiom-guidance-coverage.txt`
- [x] V5 Verify scenario "Future lints require fixtures before hard gating" with policy classification and fixture-reference checks; save output at `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/future-lint-fixture-gate.txt`
- [x] V6 Verify scenario "Functional API guidance promotes type-level state separation without opaque cleverness" with guidance coverage checks; save output at `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/functional-api-boundary-guidance.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review docs/standards and dynamic-profile guidance against https://valgrind.org/info/ and https://github.com/rust-unofficial/patterns`; input=`selected concepts from Valgrind tool-suite/run-timing model and Rust Design Patterns idiom/anti-pattern pages`; asserts=`imported concepts improve Tiger Style outcomes rather than copying generic pattern catalog entries`; evidence=`openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/source-incorporation-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: External concepts are classified before incorporation` -> `V1` -> `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/external-source-classification.txt`
- `Scenario: Dynamic profile guidance names run timing and acceptance signal` -> `V2` -> `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/dynamic-profile-guidance.txt`
- `Scenario: Heavyweight tools remain optional until a profile is selected` -> `V3` -> `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/dynamic-profile-optional-tools.txt`
- `Scenario: Rust idiom guidance maps each imported pattern to Tiger Style outcomes` -> `V4` -> `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/rust-idiom-guidance-coverage.txt`
- `Scenario: Future lints require fixtures before hard gating` -> `V5` -> `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/future-lint-fixture-gate.txt`
- `Scenario: Functional API guidance promotes type-level state separation without opaque cleverness` -> `V6` -> `openspec/changes/archive/2026-05-04-incorporate-rust-patterns-dynamic-evidence/evidence/functional-api-boundary-guidance.txt`
