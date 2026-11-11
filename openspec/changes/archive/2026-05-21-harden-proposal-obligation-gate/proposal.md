## Why

Review metrics still show repeated `omission|proposal|prompt` findings after the design and delta-spec hardening slices. The remaining failures happen before spec/design authoring: proposals omit explicit verification/evidence expectations, exact affected artifacts, concrete package/profile scope, or non-goal boundaries, so later stages cannot reliably carry those obligations forward.

## What Changes

- Harden `openspec/templates/proposal.md` with explicit proposal obligation prompts for verification/evidence, affected artifacts, concrete scope, non-goal boundaries, and downstream validation boundaries.
- Update `openspec/config.yaml` proposal-stage guidance so proposal gate review blocks missing obligation details before specs/design/tasks are drafted.
- Update the authoritative `openspec-stage-authoring` spec and add deterministic proposal-governance fixtures proving missing proposal obligations are blocked and covered proposals pass.

## Non-Goals

- Do not mutate or regenerate existing archived evidence from prior OpenSpec changes.
- Do not change production lint behavior, cargo-octet runtime behavior, or object corpus fixtures.
- Do not broaden this into tasks-stage or human-oracle hardening; those remain separate review-metric clusters.

## Capabilities

### Modified Capabilities

- `openspec-stage-authoring`: proposal template/config/spec guidance requires concrete evidence, artifact, scope, downstream, and non-goal obligations.

## Impact

- **Files / artifacts**: `openspec/templates/proposal.md`, `openspec/config.yaml`, `openspec/specs/openspec-stage-authoring/spec.md`, `tests/fixtures/openspec-proposal-governance/**`, and this change archive/evidence.
- **Ordering / dependencies**: build on the landed design/spec obligation hardening commits; keep this proposal-stage slice narrow and independent.
- **Verification / evidence**: save static helper outputs, proposal/design/tasks gate transcripts, missing/covered proposal-governance fixture gate transcripts, `openspec validate --all --strict --json`, and `git diff --check` under this change evidence/archive.
