## Why

Review metrics show omission findings are still the dominant OpenSpec review
failure mode after prior template hardening: design prompt omissions repeated 80
times, spec prompt omissions 41 times, proposal prompt omissions 33 times, and
tasks human/auto-fix omissions 23 and 16 times, as captured in
`evidence/review-metrics-omissions.txt`. The examples are concrete:
designs omit required scenario coverage, specs omit boundary cases, proposals
omit verification expectations, and tasks omit self-verifiable oracle details.

Octet already has repo-local OpenSpec templates and config rules, but the
current guidance is broad. Maintainers still need to remember common omission
classes from prior reviews instead of seeing a compact checklist before each
stage.

## What Changes

- Add omission-prevention prompts to `openspec/templates/proposal.md`,
  `openspec/templates/spec.md`, `openspec/templates/design.md`, and
  `openspec/templates/tasks.md` without changing their required heading order.
- Tighten `openspec/config.yaml` stage rules so proposal/spec/design/tasks gates
  ask authors to check repeated omission classes before review.
- Add deterministic fixture or extraction evidence proving the prompts mention
  the promoted omission classes and preserve existing template contracts.

## Non-Goals

- No rewrite of archived OpenSpec changes.
- No change to product lint behavior, CLI behavior, or consumer configuration.
- No replacement of human review; this only front-loads reviewer expectations.

## Capabilities

### New Capabilities

- `openspec-omission-prevention`: repo-local OpenSpec authoring guidance turns
  promoted repeated omission findings into visible stage checklists and proof.

### Modified Capabilities

- `openspec-stage-authoring`: proposal, delta-spec, and design guidance now
  includes repeated omission checks before implementation.
- `openspec-task-governance`: task guidance now includes repeated omission
  checks for human/oracle checkpoints, exact inputs, and explicit verification
  acceptance.

## Impact

- **Files / artifacts**: `openspec/templates/proposal.md`,
  `openspec/templates/spec.md`, `openspec/templates/design.md`,
  `openspec/templates/tasks.md`, `openspec/config.yaml`, delta specs under
  `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/specs/`, and saved
  evidence under `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/`.
- **Ordering / dependencies**: update stage-authoring and task-governance specs
  first, then templates/config, then deterministic extracts/gate transcripts.
- **Verification / evidence**: save `review-metrics-omissions.txt`, template
  excerpt artifacts for each stage, config-rule excerpt, `openspec validate`
  output, and stage gate transcripts for proposal, design, and tasks.
