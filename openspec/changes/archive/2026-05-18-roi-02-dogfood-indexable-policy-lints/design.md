## Context

`add-indexable-policy-lints` archived the nine lints and their focused UI
fixtures. This change deliberately does not alter lint semantics; it validates
operator usefulness by running the consumer-facing checker against the current
repo and preserving the result as deterministic evidence.

## Goals / Non-Goals

**Goals:**
- run the nine archived indexable-policy lints against Octet itself through a
  consumer-facing JSON path
- save raw and summarized evidence that reviewers can inspect without rerunning
- fix only obvious low-risk self-findings, or record accepted findings precisely

**Non-Goals:**
- changing the lints or their diagnostics
- forcing a clean repository if findings are intentionally accepted
- expanding to downstream consumers such as Aspen

## Decisions

### 1. Use a change-local evidence helper as the dogfood shell

**Choice:** add `evidence/run-indexable-dogfood.py` to write a temporary
`dylint.toml`, invoke the wrapped checker with JSON output, and normalize raw
findings into stable evidence files.

**Covers:** `Scenario: Dogfood run emits machine-readable finding inventory [r[indexable-policy-self-dogfood-inventory.scenario.machine-inventory]]`

**Rationale:** the helper keeps command, config, raw output, summary, and exit
status together so the evidence is replayable and reviewable.

**Alternative considered:** hand-run `cargo tigerstyle check` and paste output.
Rejected because hand commands make it easy to lose the exact enabled lint set or
exit-code semantics.

**Implementation:** use `nix run .#cargo-octet -- check --output-format json`. The helper enables only the nine target
lints by disabling all other known lints from `src/config.rs`/`LINT_INVENTORY`,
writes `dogfood-findings.jsonl`, and writes `dogfood-summary.txt` with counts by
lint and file.

**Boundary cases:** if the checker exits non-zero because findings are warnings,
record the exit code but still parse JSONL. If compilation fails, save stderr and
mark the verification task blocked instead of fabricating a clean summary.

### 2. Keep triage narrow and evidence-first

**Choice:** fix only low-risk findings that are directly reported by the dogfood
run; otherwise record accepted findings in `dogfood-triage.txt`.

**Covers:** `Scenario: Dogfood triage records fixed or accepted findings [r[indexable-policy-self-dogfood-inventory.scenario.triage-recorded]]`

**Rationale:** this slice is a product signal loop, not a broad style rewrite.
Accepted debt is useful if it names exact finding IDs and rationale.

**Alternative considered:** require zero findings before completion. Rejected
because fresh lints may intentionally surface backlog, and forcing unrelated
cleanup would hide dogfood signal.

**Implementation:** after the first inventory, inspect the summary. Make small
source/config edits only when clearly safe. Re-run the helper and write triage
notes that map final findings to `fixed` or `accepted`.

**Boundary cases:** compatibility aliases and archived evidence paths may be
intentional uses of names the lints dislike; those should be accepted with a
rationale rather than renamed opportunistically.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Dogfood run emits machine-readable finding inventory [r[indexable-policy-self-dogfood-inventory.scenario.machine-inventory]]`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/dogfood-run.txt`
  - `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/dogfood-findings.jsonl`
  - `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/dogfood-summary.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/run-indexable-dogfood.py`
- exact expected output / acceptance signal:
  - run artifact includes the exact command and exit code; findings artifact is valid JSONL when non-empty; summary includes counts grouped by lint and file
- ordering / prerequisites:
  - helper exists and archived indexable lints are present
- cleanup / postcondition:
  - no temporary dogfood config remains in the repo root

- scenario or requirement:
  - `Scenario: Dogfood triage records fixed or accepted findings [r[indexable-policy-self-dogfood-inventory.scenario.triage-recorded]]`
- artifact path:
  - `openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/dogfood-triage.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-05-18-roi-02-dogfood-indexable-policy-lints/evidence/run-indexable-dogfood.py`
- exact expected output / acceptance signal:
  - triage artifact lists every final finding as fixed or accepted with finding index, lint, path, and rationale; if final run is clean it says so explicitly
- ordering / prerequisites:
  - initial inventory summary has been inspected
- cleanup / postcondition:
  - source tree contains only intentional narrow cleanup plus evidence

## Risks / Trade-offs

- Dogfood run may be noisy -> constrain to the nine archived lints and preserve accepted debt instead of forcing broad cleanup.
- Nix/Dylint build cost may be high -> run through the existing wrapped package once per evidence step and save artifacts.
- JSON output may be empty -> still save command, exit code, and an explicit zero-finding summary.
