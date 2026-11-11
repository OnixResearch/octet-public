## Phase 1: Implementation

- [x] I1 Add `collect_local_use_sites(cx, body, hir_id) -> Vec<Span>` helper that walks a HIR `Body` and collects every `Path` segment resolving to the given `HirId` ✅ 5m
- [x] I2 Update `bool_naming` `check_local` to call the use-site collector and emit a multipart `MachineApplicable` suggestion when all spans are non-expansion; fall back to current `MaybeIncorrect` single suggestion otherwise ✅ 3m
- [x] I3 Update `negated_predicate` `check_local` to call the same collector and emit multipart `MachineApplicable` for local bindings; leave `check_item`/`check_impl_item`/`check_trait_item` at `HasPlaceholders` ✅ 3m
- [x] I4 Add `.fixed` UI fixture for `bool_naming` local-variable rename with multiple use sites ✅ 2m
- [x] I5 Add `.fixed` UI fixture for `negated_predicate` local-variable rename with multiple use sites ✅ 2m
- [x] I6 Add or update `.stderr` snapshots for updated suggestion output (multipart format, `MachineApplicable` applicability note) ✅ 2m
- [x] I7 Add macro-expansion fallback fixture showing `MaybeIncorrect` tier when declaration or use site is macro-generated ✅ 2m

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Local bool binding rename covers use sites" with `cargo test --test ui autofix_machine_applicable -- --ignored --nocapture`; save output at `openspec/changes/autofix-rename-use-sites/evidence/autofix-bool-naming.txt` ✅ 2m
- [x] V2 Verify scenario "Local negated predicate binding rename covers use sites" with `cargo test --test ui autofix_machine_applicable -- --ignored --nocapture`; save output at `openspec/changes/autofix-rename-use-sites/evidence/autofix-negated-predicate.txt` ✅ 2m
- [x] V3 Verify scenario "Macro-expanded binding falls back to review-required" with `cargo test --test ui ui -- --ignored --nocapture`; save output at `openspec/changes/autofix-rename-use-sites/evidence/macro-fallback.txt` ✅ 1m
- [x] V4 Verify scenario "Function-level negated predicate stays review-required" with `cargo test --test ui ui -- --ignored --nocapture`; save output at `openspec/changes/autofix-rename-use-sites/evidence/function-level-stays-manual.txt` ✅ 1m
- [x] V5 Verify requirement "Autofix Validation Artifacts" with saved `cargo fix` transcript in nix devshell; save output at `openspec/changes/autofix-rename-use-sites/evidence/cargo-fix-transcript.txt` ✅ 1m

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`nix develop -c bash -c 'cargo test --lib --test ui 2>&1'`; input=`current HEAD after all I-tasks`; asserts=`all existing tests still pass (no regressions from use-site collection or applicability changes)`; evidence=`openspec/changes/autofix-rename-use-sites/evidence/full-test-suite.txt` ✅ 1m

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Local bool binding rename covers use sites` -> `V1` -> `openspec/changes/autofix-rename-use-sites/evidence/autofix-bool-naming.txt`
- `Scenario: Local negated predicate binding rename covers use sites` -> `V2` -> `openspec/changes/autofix-rename-use-sites/evidence/autofix-negated-predicate.txt`
- `Scenario: Macro-expanded binding falls back to review-required` -> `V3` -> `openspec/changes/autofix-rename-use-sites/evidence/macro-fallback.txt`
- `Scenario: Function-level negated predicate stays review-required` -> `V4` -> `openspec/changes/autofix-rename-use-sites/evidence/function-level-stays-manual.txt`
- `Scenario: Review checks local-variable rename proof` -> `V5` -> `openspec/changes/autofix-rename-use-sites/evidence/cargo-fix-transcript.txt`
