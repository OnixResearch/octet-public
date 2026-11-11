## Why

Octet just archived the `add-indexable-policy-lints` work, adding nine
Indexable-derived lints plus JSON/SARIF metadata. The implementation evidence
proves focused fixtures and registration, but the next product-risk question is
whether the new lint family produces useful, machine-readable signal on Octet
itself without destabilizing the default developer loop.

Dogfooding the archived lints on this repository turns the new rules from
fixture-complete into operator-usable: we get a repeatable inventory command,
a saved finding summary, and a small documented triage boundary for any noisy or
accepted findings.

## What Changes

- Add a scoped dogfood helper under the active change evidence directory that
  runs `cargo-octet` JSON output against Octet with the nine
  indexable-policy lints enabled.
- Save deterministic evidence artifacts: raw JSON findings, summarized counts,
  accepted/fixed triage notes, and validation output.
- Apply only low-risk source/config cleanup if the dogfood run exposes obvious
  self-findings; otherwise record accepted debt with exact finding IDs.

## Non-Goals

- No new lint semantics, diagnostics, or registration changes.
- No broad repository-wide style cleanup outside the dogfood findings.
- No consumer rollout beyond this repository.

## Capabilities

### New Capabilities

- `indexable-policy-self-dogfood`: maintainers can inventory the archived
  Indexable-derived lints against Octet itself with saved machine-readable
  evidence.

### Modified Capabilities

- `consumer-integration`: JSON diagnostic output is exercised as a dogfood
  inventory input for the new lint family.

## Impact

- **Files / artifacts**: `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/`, saved evidence under its `evidence/` directory, and any narrow source/config cleanup directly tied to dogfood findings.
- **Ordering / dependencies**: reuse archived change `openspec/changes/archive/2026-05-18-add-indexable-policy-lints/`; create evidence helper first, then run inventory, then fix or classify findings.
- **Verification / evidence**: save `dogfood-run.txt`, `dogfood-findings.jsonl`, `dogfood-summary.txt`, `dogfood-triage.txt`, `openspec-validate.txt`, and proposal/design/tasks gate transcripts.
