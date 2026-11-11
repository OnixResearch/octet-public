## Phase 1: Implementation

- [x] I1 [serial] Update `openspec/templates/design.md` so each copied decision block carries the `**Obligation cross-check:**` prompt promised by this change.
- [x] I2 [depends:I1] Update `openspec/config.yaml` so the design-stage rule requires proposal/spec obligation cross-checks in addition to scenario coverage, boundary cases, and proof acceptance signals.
- [x] I3 [depends:I1] Add checked-in design-governance fixtures plus the design fixture gate helper.
- [x] I4 [depends:I1,I2,I3] Add `check-design-obligation-fixtures.py` deterministic fixture-shape helper.
- [x] I5 [depends:I1,I2] Sync the accepted `openspec/specs/openspec-stage-authoring/spec.md` design-authoring contract after validation.

## Phase 2: Deterministic verification

- [x] V1 [depends:I1,I4] Verify scenarios "Maintainer sees repeated design omission checks before implementation" and "Design author cross-checks proposal and spec obligations" with `check-design-obligation-fixtures.py --template`; save output at `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-template-obligation-fields.txt`.
- [x] V2 [depends:I3,I4] Verify scenario "Design author cross-checks proposal and spec obligations" with `check-design-obligation-fixtures.py --fixtures`; save output at `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-fixtures.txt`.
- [x] V3 [depends:I2,I4] Verify scenario "Config guidance includes promoted omission-prevention checks" with `check-design-obligation-fixtures.py --config`; save output at `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-stage-guidance.txt`.
- [x] V4 [depends:I3,I4,I5] Verify the active change with `openspec validate harden-design-obligation-gate --strict --json`; save output at `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/openspec-validate.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 [depends:V2] Human/oracle checkpoint: command=`tests/fixtures/openspec-design-governance/run-design-fixture-gate.sh design-obligation-missing-blocked`; input=`checked-in missing-obligation fixture copied to openspec/changes/design-obligation-missing-blocked`; asserts=`design-stage review reports the intentionally omitted same-branch guard suppression obligation`; evidence=`openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-missing-gate.txt`
- [x] H2 [depends:V2] Human/oracle checkpoint: command=`tests/fixtures/openspec-design-governance/run-design-fixture-gate.sh design-obligation-covered-pass`; input=`checked-in covered-obligation fixture copied to openspec/changes/design-obligation-covered-pass`; asserts=`design-stage review accepts the explicit same-branch guard suppression obligation mapping`; evidence=`openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-covered-gate.txt`
- [x] H3 [depends:V1,V2,V3,V4,H1,H2] Human/oracle checkpoint: command=`openspec_gate stage=design change=harden-design-obligation-gate`; input=`final active change proposal, delta spec, design, and tasks`; asserts=`design-stage review accepts the obligation cross-check hardening contract`; evidence=`openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-gate.txt`

## Phase 4: Final hygiene

- [x] V5 [depends:V1,V2,V3,V4,H1,H2,H3] Verify repository patch hygiene with `git diff --check`; save output at `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/git-diff-check.txt`.

## Change Status
- Status: complete
- Open tasks: none
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: Maintainer sees repeated design omission checks before implementation` -> `V1` -> `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-template-obligation-fields.txt`
- `Scenario: Design author cross-checks proposal and spec obligations` -> `V2` -> `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-obligation-fixtures.txt`
- `Scenario: Config guidance includes promoted omission-prevention checks` -> `V3` -> `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/design-stage-guidance.txt`
- `Change-level validation and patch hygiene` -> `V4,V5` -> `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/openspec-validate.txt`, `openspec/changes/archive/2026-05-21-harden-design-obligation-gate/evidence/git-diff-check.txt`
