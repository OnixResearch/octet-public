## Phase 1: Implementation

- [x] I1 Add `tools/octet-identity-sweep.scm` with a pure occurrence scanner, replacement planner, report renderer, and apply planner.
- [x] I2 Encode active-path, archive-path, replacement, allow-category, and blocked-category rules for `slotcar` / `Slotcar` identity strings.
- [x] I3 Apply the Steel-planned replacements to current docs, templates, main OpenSpec specs, Cargo/Nix surfaces, standards code, tests, UI fixtures, and generated examples.
- [x] I4 Rename standards path, package, binary, and Rust module surfaces that still carry `slotcar` identity.
- [x] I5 Remove retired `slotcar` compatibility paths or isolate retained paths behind explicit compatibility categories.
- [x] I6 Refresh generated fixtures, lock metadata, and object-corpus inventories affected by renamed package or path surfaces.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Steel dry-run inventories active slotcar references" with command=`pi steel file=tools/octet-identity-sweep.scm args='["dry-run","--format","text"]'`; input=`current repository before apply`; expect=`mode: dry-run, files_modified: 0, and every active occurrence lists path, line, match, category, replacement or reason`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-dry-run.txt`
- [x] V2 Verify scenario "Steel apply rewrites active documentation identity" with command=`pi steel file=tools/octet-identity-sweep.scm args='["apply","--format","text"]'`; input=`steel dry-run replacement plan`; expect=`mode: apply, positive rewritten file and occurrence counts, and no archive rewrites`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-apply.txt`
- [x] V3 Verify scenario "Remaining slotcar terms are explicitly allowed" with command=`pi steel file=tools/octet-identity-sweep.scm args='["verify","--format","text"]'`; input=`repository after apply and generated-artifact refresh`; expect=`blocked: 0 and unapproved_active_slotcar: 0`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-final-verify.txt`
- [x] V4 Verify scenario "Archived evidence remains historical" with command=`pi steel file=tools/octet-identity-sweep.scm args='["verify","--format","text"]'`; input=`openspec/changes/archive/** and cairn/archive/** scan rows`; expect=`archive rows are classified allowed-archive and apply report names no archived rewrites`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-final-verify.txt`
- [x] V5 Verify scenario "Steel sweep handles command and package surfaces" with command=`nix develop -c bash -c 'cargo test -p cargo-octet octet_identity_surfaces -- --nocapture && cargo test -p octet-standards octet_identity_surfaces -- --nocapture'`; input=`renamed Cargo/Nix/standards surfaces after Steel apply`; expect=`focused identity-surface tests pass for Octet command, package, Nix, and structured-output strings`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/focused-checks.txt`
- [x] V6 Verify scenario "Compatibility retirement or allowlist is explicit" with command=`pi steel file=tools/octet-identity-sweep.scm args='["compatibility-report","--format","text"]'`; input=`repository after final verify`; expect=`each retained slotcar compatibility row has rationale and focused test, and each retired row names the Octet replacement`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/compatibility-decisions.txt`
- [x] V7 Verify scenario "Octet config and command paths stay functional" with command=`nix develop -c bash -c 'cargo test -p cargo-octet octet_config_and_command_paths -- --nocapture && git diff --check'`; input=`renamed config parser, CLI parser, docs, and templates`; expect=`Octet config/command tests pass and diff whitespace check is clean`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`pi steel file=tools/octet-identity-sweep.scm args='["compatibility-report","--format","text"]'`; input=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/compatibility-decisions.txt`; asserts=`retained or retired slotcar compatibility surfaces match the intended breaking-change boundary`; evidence=`openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/compatibility-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Steel dry-run inventories active slotcar references` -> `V1` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-dry-run.txt`
- `Scenario: Steel apply rewrites active documentation identity` -> `V2` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-apply.txt`
- `Scenario: Remaining slotcar terms are explicitly allowed` -> `V3` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-final-verify.txt`
- `Scenario: Archived evidence remains historical` -> `V4` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/steel-final-verify.txt`
- `Scenario: Steel sweep handles command and package surfaces` -> `V5` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/focused-checks.txt`
- `Scenario: Compatibility retirement or allowlist is explicit` -> `V6` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/compatibility-decisions.txt`
- `Scenario: Octet config and command paths stay functional` -> `V7` -> `openspec/changes/archive/2026-06-11-mechanical-slotcar-to-octet-sweep/evidence/focused-checks.txt`
