## Phase 1: Machine-applicable fixes

- [x] `compound_assertion`: emit a `multipart_suggestion` only for validated top-level `assert!(a && b)` cases
- [x] Add `.fixed` proof for every `compound_assertion` pattern marked `MachineApplicable` (`ui_tests/compound_assertion_machine_applicable.fixed`, `ui_tests/compound_assertion_indented.fixed`)
- [x] Add an indentation-sensitive `compound_assertion` fixture proving the multipart rewrite preserves source indentation in snapshots or `.fixed` output
- [x] Add guardrail fixtures proving top-level `assert!(a || b)` and message-bearing `assert!(a && b, "...")` stay out of `MachineApplicable`

## Phase 2: Review-required (`MaybeIncorrect`) suggestions

- [x] `bool_naming`: declaration-scoped rename suggestion with review-required applicability unless use-site propagation lands in the same change
- [x] `acronym_style`: declaration-scoped rename suggestion with review-required applicability unless use-site propagation lands in the same change
- [x] `unchecked_division`: suggest `assert!(b != 0);` insertion
- [x] `raw_arithmetic_overflow`: suggest checked or saturating arithmetic replacements
- [x] `unchecked_narrowing`: suggest `u32::try_from(value)?` replacement
- [x] `platform_dependent_cast`: suggest `usize::try_from(value)?` replacement
- [x] Add or update UI fixtures so `.stderr` snapshots prove the suggestion text and review-required applicability for every Phase 2 lint

## Phase 3: Placeholder (`HasPlaceholders`) suggestions

- [x] `numeric_units`: suggest a unit-bearing rename with explicit placeholder choices
- [x] `unjustified_allow`: suggest a stub justification comment that still requires maintainer-written rationale
- [x] Add or update UI fixtures so `.stderr` snapshots prove placeholder suggestion text for every Phase 3 lint

## Phase 4: Validation artifacts

- [x] Save ignored UI proof for the full rollout fixture set under `openspec/changes/archive/2026-04-16-autofix-suggestions/evidence/cargo-test-ui-ignored.txt`, with each exercised autofix fixture name printed and the covered snapshot set made explicit in the saved artifact
- [x] Save the supported nix-devshell `cargo fix` proof transcript under `openspec/changes/archive/2026-04-16-autofix-suggestions/evidence/cargo-fix-proof.txt`, including the exact proof command and machine-applicable target exercised
- [x] Update all `.stderr` expectations to include suggestion text and applicability notes
- [x] Verify every `.fixed` proof file compiles without warnings via the ignored `autofix_machine_applicable` rail in `tests/ui.rs`
- [x] Add a guardrail check in `src/lib.rs` proving the diagnostics-only and unchanged out-of-rollout lints named in `specs/autofix/spec.md` stay outside autofix artifacts and implementation scope for this change
