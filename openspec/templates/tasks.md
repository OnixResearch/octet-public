<!-- Copy this file into openspec/changes/<change>/tasks.md and replace the placeholders. -->
<!-- Use this template for active changes under openspec/changes/<change>/; archived changes keep their historical task shape. -->
<!-- Omission-prevention checklist: keep implementation separate from verification/evidence, give every added or modified scenario its own coverage row, make each V<n> task name the exact scenario, command/helper, required input/artifact, evidence path, and acceptance signal, and make each H<n> oracle checkpoint self-verifiable with exact command=, input=, asserts=, and evidence= fields. -->

## Task ID rules

- `I<n>` — one implementation claim only. Do not combine code changes with verification runs, saved evidence, or transcript capture in the same checkbox. Split that work into a separate `V<n>` or `H<n>` task.
- Treat an `I<n>` checkbox as invalid if its text includes verification/evidence markers such as `verify`, `save`, `capture`, `evidence=`, or a repo-relative `evidence/` path.
- `V<n>` — one deterministic verification claim for exactly one delta-spec scenario or one requirement-level invariant that has no scenario heading. Name the exact scenario, command/helper, required input or artifact, saved evidence path, and expected PASS output, diagnostic, schema field, or other acceptance signal.
- Treat a `V<n>` checkbox as underspecified if it can be satisfied by a broader check that omits the scenario-specific behavior, required artifact, negative control, or continued-warning case.
- Before marking a change ready, read the delta specs and ensure every added or modified `Scenario:` heading appears in `## Verification Coverage` exactly once. Shared artifacts still need separate scenario rows.
- For Valence object-evidence changes touching `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, THIR/MIR fixture inventories, or `cargo-octet/tests/fixtures/object-corpus/*`, prepare an unselected v2 candidate if source/receipt bindings are stale, require independent review before any approved reference refresh, and include a V-task for the EXISTING `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L` rail before aggregate closeout; do not replace it with a new gate or regenerate an approved reference to obtain PASS.
- Treat regenerated Valence object receipts as **unselected candidates** until a separate owner-reviewed reference binds the exact sorted selected source paths and byte hashes, every internal/expanded/fixed-fixture receipt and taxonomy hash, and the reviewed Valence revision. Even a one-byte comment edit that preserves the token object-set hash invalidates the source-byte binding. An absent, stale, or `PENDING` reference keeps existing named object rails fail-closed; two self-produced matching receipts or binaries cannot satisfy a source-bound freshness V-task. Preserve historical v1 artifacts rather than silently re-pinning them to v2.
- `H<n>` — one human/oracle checkpoint using the exact field layout shown in `H1` below.
- Treat an `H<n>` checkbox as invalid if any of `command=`, `input=`, `asserts=`, or `evidence=` is missing.
- Treat an `H<n>` checkbox as underspecified if `command=` is a broad workflow instead of an exact command/helper, `input=` omits the pinned fixture or prerequisite artifact, `asserts=` bundles multiple human claims, or `evidence=` omits the saved review artifact path.

## Phase 1: Implementation

- [ ] I1 Implement <one code or document change>
- [ ] I2 Update <one follow-on code or document change>

Bad: `- [ ] I3 update parser and save cargo-test-lib.txt`
Good: `- [ ] I3 update parser` plus `- [ ] V4 save cargo-test-lib.txt`

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "<Scenario heading>" with command=`<exact test/helper>`; input=`<fixture, artifact, or source path>`; expect=`<PASS output, diagnostic, schema field, or acceptance signal>`; evidence=`openspec/changes/<change>/evidence/<artifact>.txt`
- [ ] V2 Verify requirement "<Requirement heading>" with command=`<exact test/helper>`; input=`<fixture, artifact, or source path>`; expect=`<PASS output, diagnostic, schema field, or acceptance signal>`; evidence=`openspec/changes/<change>/evidence/<artifact>.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`<exact command or helper path>`; input=`<pinned revision, fixture, or prerequisite artifact>`; asserts=`<single human claim>`; evidence=`openspec/changes/<change>/evidence/<artifact>.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, V1, V2, H1
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: <Scenario heading>` -> `V1` -> `openspec/changes/<change>/evidence/<artifact>.txt`
- `Requirement: <Requirement heading>` -> `V2` -> `openspec/changes/<change>/evidence/<artifact>.txt`

Use one coverage row for every added or modified scenario heading.
Use `Requirement:` rows only when a requirement has zero scenario headings.
Multiple rows may point at the same artifact, but each scenario still needs its
own row.

Bad: `Scenario: A and B` -> `V3` -> `openspec/changes/<change>/evidence/shared.txt`
Good:
- `Scenario: A` -> `V3` -> `openspec/changes/<change>/evidence/shared.txt`
- `Scenario: B` -> `V4` -> `openspec/changes/<change>/evidence/shared.txt`

Bad: `- [ ] V5 Unit tests for config parsing with new fields` when the spec requires absent-field defaults.
Good: `- [ ] V5 Verify scenario "Absent disabled_lints defaults to empty" with command=`cargo test -p cargo-octet config_absent_defaults`; input=`ui_tests/no_unwrap_default/`; expect=`PASS absent disabled_lints and lint_levels default empty`; evidence=`openspec/changes/<change>/evidence/config-absent-defaults.txt``
