## Phase 1: Implementation

- [x] I1 [serial] Patch `openspec/templates/proposal.md` with proposal obligation prompts for exact evidence, affected artifacts, concrete scope, downstream validation boundaries, and non-goals.
- [x] I2 [serial] [depends:I1] Patch `openspec/config.yaml` and `openspec/specs/openspec-stage-authoring/spec.md` to make the proposal-stage obligation contract authoritative.
- [x] I3 [serial] [depends:I2] Add proposal-governance fixture pair and runner under `tests/fixtures/openspec-proposal-governance/`.
- [x] I4 [serial] [depends:I3] Add deterministic evidence helper under this change evidence directory.

## Phase 2: Verification

- [x] V1 [depends:I4] Run static helper checks for template/config/fixtures and save outputs under `openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/`.
- [x] V2 [depends:I4] Run `openspec validate harden-proposal-obligation-gate --strict --json` and save evidence.
- [x] V3 [depends:V1] Run missing proposal fixture gate and save `evidence/proposal-obligation-missing-gate.txt`.
- [x] V4 [depends:V1] Run covered proposal fixture gate and save `evidence/proposal-obligation-covered-gate.txt`.
- [x] V5 [depends:V2] Run proposal gate for this change and save `evidence/proposal-gate.txt`.
- [x] V6 [depends:V5] Run design gate for this change and save `evidence/design-gate.txt`.
- [x] V7 [depends:V6] Run tasks gate for this change and save `evidence/tasks-gate.txt`.
- [x] A1 [depends:V7] Run `openspec archive harden-proposal-obligation-gate -y`, rewrite active-change paths to the archive path, and record archive closeout evidence.
- [x] V8 [depends:A1] After archive, run `openspec validate --all --strict --json` and `git diff --check`; save evidence in the archive.
- [x] V9 [depends:A1] After archive, rerun the archived evidence helper and active-path check; save archive closeout evidence.

## Human / Oracle Checkpoints

- [x] H1 [depends:V3,V4] command=`tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh proposal-obligation-missing-blocked && tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh proposal-obligation-covered-pass`; input=`tests/fixtures/openspec-proposal-governance/**`; asserts=`missing fixture blocks target proposal omission and covered fixture passes`; evidence=`openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/proposal-obligation-*-gate.txt`
- [x] H2 [depends:V5,V6,V7] command=`pi --no-session --thinking minimal -e ~/.pi/agent/extensions/openspec-gates.ts -p '<openspec_gate stage for harden-proposal-obligation-gate>'`; input=`openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/{proposal.md,design.md,tasks.md,specs/**}`; asserts=`proposal/design/tasks gates pass`; evidence=`openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/{proposal-gate.txt,design-gate.txt,tasks-gate.txt}`
- [x] H3 [depends:V8,V9] command=`openspec validate --all --strict --json && git diff --check`; input=`openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/`; asserts=`archive synced, helper works from archive, no former active-change path references remain`; evidence=`openspec/changes/archive/2026-05-21-harden-proposal-obligation-gate/evidence/{openspec-validate.txt,git-diff-check.txt,archived-helper-rerun.txt,archive-path-rewrite.txt}`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

| Scenario | Verification | Evidence |
| --- | --- | --- |
| Maintainer sees repeated proposal omission checks before drafting | Static template/config helper checks and proposal gate | `evidence/proposal-template-obligation-fields.txt`, `evidence/proposal-stage-guidance.txt`, `evidence/proposal-gate.txt` |
| Proposal author records exact scope and downstream validation boundaries | Covered proposal-governance fixture plus design/proposal gates | `evidence/proposal-obligation-covered-gate.txt`, `evidence/design-gate.txt` |
| Proposal governance fixtures prove blocked and covered obligations | Missing and covered fixture runner outputs | `evidence/proposal-obligation-missing-gate.txt`, `evidence/proposal-obligation-covered-gate.txt` |
| Config guidance includes promoted omission-prevention checks | Static config helper and final OpenSpec validation | `evidence/proposal-stage-guidance.txt`, `evidence/openspec-validate.txt` |
