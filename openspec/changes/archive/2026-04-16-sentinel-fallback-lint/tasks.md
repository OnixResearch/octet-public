## Phase 1: Implementation

- [x] I1 Implement `sentinel_fallback` as a late safety lint over `Option` / `Result` `unwrap_or` and `unwrap_or_else` with integer `MIN` / `MAX` sentinels
- [x] I2 Register `sentinel_fallback` in `src/lib.rs`, add diagnostic metadata in `src/diagnostics.rs`, and add ignored UI fixtures plus snapshots for positive and negative cases

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Fallible conversion masked by max sentinel" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-fallible-max.txt`
- [x] V2 Verify scenario "Optional index masked by max sentinel" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-option-max.txt`
- [x] V3 Verify scenario "Side-effect closure masked by min sentinel" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-closure-min.txt`
- [x] V4 Verify scenario "Explicit branch stays quiet" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-explicit-branch.txt`
- [x] V5 Verify scenario "Non-extreme default stays quiet" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-non-extreme-default.txt`
- [x] V6 Verify scenario "Non-extreme closure default stays quiet" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-non-extreme-closure.txt`
- [x] V7 Verify scenario "Test-context sentinel fallback stays quiet" with ignored UI coverage; save output at `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-test-context.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`nix develop -c bash -c 'cargo test --test ui ui -- --ignored --nocapture'`; input=`sentinel_fallback UI fixture plus the reviewed Aspen example shape format!("recent_{}", u64::try_from(command_index).unwrap_or(u64::MAX))`; asserts=`diagnostic clearly says the sentinel hides absence/failure and points maintainers toward explicit branching, propagation, or tagged fallback text`; evidence=`openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-human-review.txt`

## Verification Coverage

- `Scenario: Fallible conversion masked by max sentinel` -> `V1` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-fallible-max.txt`
- `Scenario: Optional index masked by max sentinel` -> `V2` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-option-max.txt`
- `Scenario: Side-effect closure masked by min sentinel` -> `V3` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-closure-min.txt`
- `Scenario: Explicit branch stays quiet` -> `V4` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-explicit-branch.txt`
- `Scenario: Non-extreme default stays quiet` -> `V5` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-non-extreme-default.txt`
- `Scenario: Non-extreme closure default stays quiet` -> `V6` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-non-extreme-closure.txt`
- `Scenario: Test-context sentinel fallback stays quiet` -> `V7` -> `openspec/changes/archive/2026-04-16-sentinel-fallback-lint/evidence/ui-test-context.txt`
