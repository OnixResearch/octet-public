## Phase 1: Implementation

- [x] I1 Migrate standards source material into `docs/standards/source/` with Tiger Style naming.
- [x] I2 Add `crates/tigerstyle-standards` pure checker core and CLI shell.
- [x] I3 Add `standards/policies/` Nickel registry, contracts, minimum coverage, and positive/negative registry fixtures.
- [x] I4 Add `standards/check-fixtures/` positive and negative fixtures for suppressions, architecture, and bounded resources.
- [x] I5 Wire Nix apps and checks for `tigerstyle-standards`, `tigerstyle-policy-export`, `tigerstyle-policy-validate`, `policy-registry`, and `standards-fixtures`.
- [x] I6 Update README, contributor guide, standards contract, and DevTools rail audit to name the Tiger Style standards seam.
- [x] I7 Implement remaining architecture/public API/external-domain leakage checks after the foundation lands, with thick-root and public-API-budget fixtures kept separate.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Legacy project names are absent from Tiger Style standards artifacts" with ripgrep; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/no-legacy-names.txt`.
- [x] V2 Verify scenarios "Valid Nickel policy exports for tools", "Invalid policy fails before checks run", and "Missing traceability is rejected" with `nix run .#tigerstyle-policy-validate`; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/policy-validate.txt`.
- [x] V3 Verify scenarios "Blocking hard rule fails deterministically", "Advisory rule reports without blocking", "Hard rule has fixture evidence", "Narrow suppression is accepted", and "Blanket suppression is rejected" with `nix develop -c bash -c 'export CARGO_TARGET_DIR=target/verify-standards; cargo test -p tigerstyle-standards'`; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`.
- [x] V4 Verify scenario "Fixture regression blocks CI" with flake `policy-registry` and `standards-fixtures`; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/flake-standards-checks.txt`.
- [x] V5 Verify the future scenario "Candidate rule with weak signal becomes advisory" after the remaining policy classification evidence is refreshed, including the exported `classification_rationale` that records why it is not blocking yet; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/advisory-classification.txt`.
- [x] V6 Verify scenario "Local-first standards verification succeeds" with flake `fmt`, `clippy`, `nextest`, and `ui` seams for formatting, Clippy denial, workspace checks, and Tiger Style lint checks; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/verification-rails.txt`.
- [x] V7 Verify OpenSpec artifact validity with `openspec validate codify-tigerstyle-standards --strict`; save output at `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/openspec-validate.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`AskClaude read-only staged-change review`; input=`staged Tiger Style standards migration`; asserts=`Dylint lint engine and standards policy checker remain separated by crate/command seams`; evidence=`openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/secondary-review.txt`.

## Deferred Follow-up

Later-wave embedded/resilience checks, orchestration/idiom/test/telemetry/fitness/safety-evidence reports, and a final single-command standards entrypoint are intentionally out of scope for this change per the proposal Non-Goals. They need separate OpenSpec changes before implementation.

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Codifiable source becomes policy` -> `V2` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Human-only guidance stays explicit` -> `V5` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/advisory-classification.txt`
- `Scenario: Missing traceability is rejected` -> `V2` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Valid Nickel policy exports for tools` -> `V2` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Invalid policy fails before checks run` -> `V2` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/policy-validate.txt`
- `Scenario: Blocking hard rule fails deterministically` -> `V3` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Advisory rule reports without blocking` -> `V3` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Candidate rule with weak signal becomes advisory` -> `V5` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/advisory-classification.txt`
- `Scenario: Hard rule has fixture evidence` -> `V3` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Narrow suppression is accepted` -> `V3` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Blanket suppression is rejected` -> `V3` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/standards-tests.txt`
- `Scenario: Local-first standards verification succeeds` -> `V6` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/verification-rails.txt`
- `Scenario: Fixture regression blocks CI` -> `V4` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/flake-standards-checks.txt`
- `Scenario: Legacy project names are absent from Tiger Style standards artifacts` -> `V1` -> `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/no-legacy-names.txt`
