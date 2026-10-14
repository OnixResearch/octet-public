## Phase 1: Implementation

- [x] [serial] I1 Patch `openspec/templates/spec.md` with an obligation cross-check prompt.
- [x] [depends:I1] I2 Patch `openspec/config.yaml` spec-stage guidance for proposal/design obligation carry-forward.
- [x] [depends:I1] I3 Update `openspec/specs/openspec-stage-authoring/spec.md` through the delta spec.
- [x] [parallel] I4 Add spec-governance fixture pair and proposal-gate runner.
- [x] [depends:I1] I5 Add static evidence helper for template/config/fixture assertions.

## Phase 2: Deterministic verification

- [x] [depends:I5] V1 Verify scenario "Spec author cross-checks proposal and design obligations" with static helper; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-template-obligation-fields.txt`.
- [x] [depends:I2,I5] V2 Verify scenario "Config guidance includes promoted omission-prevention checks" with static helper; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-stage-guidance.txt`.
- [x] [depends:I4,I5] V3 Verify fixture coverage with static helper; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-fixtures.txt`.
- [x] [depends:I2,I4] V4 Verify missing-obligation fixture blocks at proposal gate; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-missing-gate.txt`.
- [x] [depends:I2,I4] V5 Verify covered-obligation fixture passes at proposal gate; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-covered-gate.txt`.
- [x] [depends:I3] V6 Verify OpenSpec strict validation; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/openspec-validate.txt`.
- [x] [depends:I1,I2,I3,I4,I5] V7 Verify whitespace with `git diff --check`; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/git-diff-check.txt`.

## Phase 3: Human/oracle checkpoints

- [x] [depends:V1,V2,V3,V4,V5,V6,V7] H1 Human/oracle checkpoint: command=`pi --no-session --thinking minimal -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call the openspec_gate tool with stage=proposal and change=harden-spec-obligation-gate. Print the full result."`; input=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/{proposal.md,specs/**/*.md}`; asserts=`proposal gate accepts the active change after spec-obligation prompts and fixtures are specified`; evidence=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/proposal-gate.txt`
- [x] [depends:H1] H2 Human/oracle checkpoint: command=`pi --no-session --thinking minimal -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call the openspec_gate tool with stage=design and change=harden-spec-obligation-gate. Print the full result."`; input=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/design.md`; asserts=`design gate accepts the obligation cross-check design`; evidence=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/design-gate.txt`
- [x] [depends:H2,V1,V2,V3,V4,V5,V6,V7] H3 Human/oracle checkpoint: command=`pi --no-session --thinking minimal -e ~/.pi/agent/extensions/openspec-gates.ts -p "Call the openspec_gate tool with stage=tasks and change=harden-spec-obligation-gate. Print the full result."`; input=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/tasks.md`; asserts=`tasks gate accepts task/evidence coverage`; evidence=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/tasks-gate.txt`

## Phase 4: Archive closeout

- [x] [depends:H3] H4 Human/oracle checkpoint: command=`openspec archive harden-spec-obligation-gate -y`; input=`former active change directory for harden-spec-obligation-gate`; asserts=`archive directory exists at openspec/changes/archive/2026-05-21-harden-spec-obligation-gate and the former active change path is removed`; evidence=`openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/archive-closeout.txt`
- [x] [depends:H4] V8 Verify archived helper reruns and saved evidence paths from archived location; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/archived-helper-rerun.txt`.
- [x] [depends:V8] V9 Verify no former active-change path references remain in `openspec/`; save output at `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/archive-path-rewrite.txt`.

## Change Status
- Status: complete
- Open tasks: none
- Archive closeout: completed after OpenSpec archive; active evidence paths were rewritten to `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/...`, archived helper evidence was rerun, and former active-change path references were checked before commit.
## Verification Coverage

- `Scenario: Spec author cross-checks proposal and design obligations` -> `V1` -> `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-template-obligation-fields.txt`
- `Scenario: Config guidance includes promoted omission-prevention checks` -> `V2` -> `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-stage-guidance.txt`
- `Scenario: Maintainer sees repeated delta-spec omission checks before review` -> `V3` -> `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-fixtures.txt`
- `Scenario: Spec governance fixtures prove blocked and covered obligations` -> `V4`/`V5` -> `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-missing-gate.txt` and `openspec/changes/archive/2026-05-21-harden-spec-obligation-gate/evidence/spec-obligation-covered-gate.txt`
